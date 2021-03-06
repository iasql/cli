use ascii_table::{AsciiTable, Column};
use indicatif::ProgressBar;
use serde::{Deserialize, Serialize};
use serde_ini;
use serde_json::json;

use std::collections::HashMap;
use std::error::Error;
use std::fmt::Display;
use std::fs::{write, OpenOptions};
use std::io::BufReader;
use std::process::exit;

use crate::dialoguer as dlg;
use crate::http::{get_v1, post_v1};

#[derive(Deserialize, Debug, Clone, Serialize)]
pub struct AWSCLICredentials {
  aws_access_key_id: String,
  aws_secret_access_key: String,
}

#[derive(Deserialize, Debug, Clone, Serialize)]
pub struct NewDbResponse {
  id: String,
  alias: String,
  user: String,
  password: String,
}

#[derive(Deserialize, Debug, Clone, Serialize)]
#[allow(non_snake_case)]
pub struct PlanMeta {
  columns: Vec<String>,
  records: Vec<Vec<String>>,
}
#[derive(Deserialize, Debug, Clone, Serialize)]
#[allow(non_snake_case)]
pub struct PlanResponse {
  iasqlPlanVersion: i32,
  toCreate: HashMap<String, PlanMeta>,
  toUpdate: HashMap<String, PlanMeta>,
  toReplace: HashMap<String, PlanMeta>,
  toDelete: HashMap<String, PlanMeta>,
}

// TODO load regions at startup based on aws services and schema since not all regions support all services.
// Currently manually listing ec2 regions that do not require opt-in status in alphabetical order
// https://docs.aws.amazon.com/AWSEC2/latest/UserGuide/using-regions-availability-zones.html
fn get_aws_regions() -> Vec<String> {
  return vec![
    "ap-northeast-1".to_string(),
    "ap-northeast-2".to_string(),
    "ap-northeast-3".to_string(),
    "ap-south-1".to_string(),
    "ap-southeast-1".to_string(),
    "ap-southeast-2".to_string(),
    "ca-central-1".to_string(),
    "eu-central-1".to_string(),
    "eu-north-1".to_string(),
    "eu-west-1".to_string(),
    "eu-west-2".to_string(),
    "eu-west-3".to_string(),
    "sa-east-1".to_string(),
    "us-east-1".to_string(),
    "us-east-2".to_string(),
    "us-west-1".to_string(),
    "us-west-2".to_string(),
  ];
}

fn get_aws_cli_creds() -> Result<HashMap<String, AWSCLICredentials>, String> {
  let home = std::env::var("HOME").unwrap();
  let file_name = &format!("{}/.aws/credentials", home);
  let file = OpenOptions::new().read(true).open(file_name);
  if let Err(err) = file {
    return Err(err.to_string());
  }
  let reader = BufReader::new(file.unwrap());
  match serde_ini::from_bufread(reader) {
    Ok(creds) => Ok(creds),
    Err(err) => Err(err.to_string()),
  }
}

fn get_server() -> &'static str {
  let default = if cfg!(debug_assertions) {
    "local"
  } else {
    "production"
  };
  let env = std::env::var("IASQL_ENV").unwrap_or(default.to_string());
  match env.as_str() {
    "local" => "127.0.0.1:5432",
    _ => "db.iasql.com",
  }
}

pub async fn get_or_select_db(db_opt: Option<&str>) -> String {
  let dbs = get_dbs(true).await;
  if db_opt.is_none() {
    let selection = if dbs.len() > 1 {
      dlg::select_with_default("Pick hosted IaSQL db", &dbs, 0)
    } else {
      // if only one db, skip selection
      println!(
        "{} {} {} {}",
        dlg::success_prefix(),
        dlg::bold("IaSQL db"),
        dlg::divider(),
        dlg::green(&dbs[0]),
      );
      0
    };
    let db = &dbs[selection];
    db.clone()
  } else {
    let db = db_opt.unwrap();
    if !dbs.contains(&db.to_owned()) {
      eprintln!(
        "{} {} {} {}",
        dlg::err_prefix(),
        dlg::bold("Nonexistent hosted db"),
        dlg::divider(),
        dlg::red(db)
      );
      exit(1);
    }
    db.to_string()
  }
}

async fn get_or_input_db(db_opt: Option<&str>) -> String {
  let db = if db_opt.is_none() {
    dlg::optional_input("Optional db name")
  } else {
    db_opt.unwrap().to_string()
  };
  let dbs = get_dbs(false).await;
  if dbs.contains(&db.to_owned()) {
    eprintln!(
      "{} {} {} {}",
      dlg::err_prefix(),
      dlg::bold("Name already in use by another hosted db"),
      dlg::divider(),
      dlg::red(&db)
    );
    exit(1);
  }
  db
}

pub fn get_or_input_arg(arg_opt: Option<&str>, in_title: &str) -> String {
  if arg_opt.is_none() {
    dlg::input(in_title)
  } else {
    arg_opt.unwrap().to_string()
  }
}

async fn get_dbs(exit_if_none: bool) -> Vec<String> {
  let resp = get_v1("db/list").await;
  let res = match &resp {
    Ok(r) => r,
    Err(e) => {
      eprintln!(
        "{} {} {} {}",
        dlg::err_prefix(),
        dlg::bold("Failed to get all hosted dbs"),
        dlg::divider(),
        e.message
      );
      exit(1);
    }
  };
  let dbs: Vec<String> = serde_json::from_str(res).unwrap();
  if exit_if_none && dbs.len() == 0 {
    println!(
      "{} {}",
      dlg::warn_prefix(),
      dlg::bold("No hosted db to manage a cloud account has been created")
    );
    exit(0);
  }
  dbs
}

pub async fn export(db: &str, dump_file: String, data_only: bool) {
  let sp = ProgressBar::new_spinner();
  sp.enable_steady_tick(10);
  sp.set_message("Export in progress");
  let df = if !dump_file.ends_with(".sql") {
    format!("{}.sql", dump_file)
  } else {
    dump_file
  };
  let body = json!({
    "dbAlias": db,
    "dataOnly": data_only,
  });
  let resp = post_v1("db/export/", body).await;
  sp.finish_and_clear();
  match &resp {
    Ok(r) => {
      let export_response: String = serde_json::from_str(r).unwrap();
      match write(df, export_response) {
        Ok(_) => println!("{} {}", dlg::success_prefix(), dlg::bold("Done")),
        Err(e) => {
          eprintln!(
            "{} {} {} {} {} {}",
            dlg::err_prefix(),
            dlg::bold("Failed to export hosted db"),
            dlg::divider(),
            dlg::red(db),
            dlg::divider(),
            e
          );
          exit(1);
        }
      };
    }
    Err(e) => {
      eprintln!(
        "{} {} {} {} {} {}",
        dlg::err_prefix(),
        dlg::bold("Failed to export hosted db"),
        dlg::divider(),
        dlg::red(db),
        dlg::divider(),
        e.message
      );
      exit(1);
    }
  };
}

pub async fn list() {
  let dbs = get_dbs(true).await;
  let mut table = AsciiTable::default();
  table.max_width = 140;
  let column = Column {
    header: "Hosted Database Name".into(),
    ..Column::default()
  };
  table.columns.insert(0, column);
  let mut db_data: Vec<Vec<&dyn Display>> = vec![];
  for db in dbs.iter() {
    let mut row: Vec<&dyn Display> = Vec::new();
    row.push(db);
    db_data.push(row);
  }
  table.print(db_data);
}

pub async fn remove(db: &str, noninteractive: bool) {
  if !noninteractive {
    let removal = dlg::confirm_with_default("Press enter to confirm removal", true);
    if !removal {
      println!(
        "{} {} {} {}",
        dlg::warn_prefix(),
        dlg::bold("Did not remove db"),
        dlg::divider(),
        dlg::yellow(db)
      );
      exit(0);
    }
  }
  let resp = get_v1(&format!("db/remove/{}", db)).await;
  match &resp {
    Ok(_) => println!("{} {}", dlg::success_prefix(), dlg::bold("Done")),
    Err(e) => {
      eprintln!(
        "{} {} {} {} {} {}",
        dlg::err_prefix(),
        dlg::bold("Failed to remove db"),
        dlg::divider(),
        dlg::red(db),
        dlg::divider(),
        e.message
      );
      exit(1);
    }
  };
}

fn emit_plan_segment(crupde: HashMap<String, PlanMeta>, mode_str: &str) {
  for key in crupde.keys() {
    let meta = crupde.get(key).unwrap();
    let count = meta.records.len();
    let record_text = if count == 1 { "record" } else { "records" };
    println!(
      "{} has {} {} to {}",
      dlg::bold(key),
      dlg::bold(&format!("{}", count)),
      dlg::bold(record_text),
      mode_str,
    );
    let mut table = AsciiTable::default();
    table.max_width = 160;
    for (i, column) in meta.columns.iter().enumerate() {
      table.columns.insert(
        i,
        Column {
          header: column.to_string(),
          ..Column::default()
        },
      );
    }
    table.print(meta.records.clone());
  }
}

fn maybe_planned_nothing(plan_response: &PlanResponse) {
  if plan_response.toCreate.keys().len() == 0
    && plan_response.toUpdate.keys().len() == 0
    && plan_response.toReplace.keys().len() == 0
    && plan_response.toDelete.keys().len() == 0
  {
    println!(
      "{} No difference detected between hosted db and cloud account",
      dlg::warn_prefix(),
    );
  }
}

pub async fn plan(db: &str, noninteractive: bool) {
  if !noninteractive {
    let confirmation = dlg::confirm_with_default("Press enter to confirm plan", true);
    if !confirmation {
      println!(
        "{} {} {} {}",
        dlg::warn_prefix(),
        dlg::bold("Did not run plan on db"),
        dlg::divider(),
        dlg::yellow(db)
      );
      exit(0);
    }
  }
  let sp = ProgressBar::new_spinner();
  sp.enable_steady_tick(10);
  sp.set_message("Plan in progress");
  // call apply with dryRun set to true
  let body = json!({
    "dbAlias": db,
    "dryRun": true,
  });
  let resp = post_v1("db/apply/", body).await;
  sp.finish_and_clear();
  match &resp {
    Ok(r) => {
      let plan_response: PlanResponse = serde_json::from_str(r).unwrap();
      maybe_planned_nothing(&plan_response);
      emit_plan_segment(plan_response.toCreate, &dlg::green("create").to_string());
      emit_plan_segment(plan_response.toUpdate, &dlg::yellow("update").to_string());
      emit_plan_segment(
        plan_response.toReplace,
        &dlg::magenta("replace").to_string(),
      );
      emit_plan_segment(plan_response.toDelete, &dlg::red("delete").to_string());
    }
    Err(e) => {
      eprintln!(
        "{} {} {} {} {} {}",
        dlg::err_prefix(),
        dlg::bold("Failed to run plan on db"),
        dlg::divider(),
        dlg::red(db),
        dlg::divider(),
        e.message
      );
      exit(1);
    }
  };
}

pub async fn apply(db: &str, noninteractive: bool) {
  if !noninteractive {
    let confirmation = dlg::confirm_with_default("Press enter to confirm apply", true);
    if !confirmation {
      println!(
        "{} {} {} {}",
        dlg::warn_prefix(),
        dlg::bold("Did not run apply on db"),
        dlg::divider(),
        dlg::yellow(db)
      );
      exit(0);
    }
  }
  let sp = ProgressBar::new_spinner();
  sp.enable_steady_tick(10);
  sp.set_message("Apply in progress");
  let body = json!({
    "dbAlias": db,
  });
  let resp = post_v1("db/apply/", body).await;
  sp.finish_and_clear();
  match &resp {
    Ok(r) => {
      let plan_response: PlanResponse = serde_json::from_str(r).unwrap();
      maybe_planned_nothing(&plan_response);
      emit_plan_segment(plan_response.toCreate, &dlg::green("create").to_string());
      emit_plan_segment(plan_response.toUpdate, &dlg::yellow("update").to_string());
      emit_plan_segment(
        plan_response.toReplace,
        &dlg::magenta("replace").to_string(),
      );
      emit_plan_segment(plan_response.toDelete, &dlg::red("delete").to_string());
      println!("{} {}", dlg::success_prefix(), dlg::bold("Done"));
    }
    Err(e) => {
      eprintln!(
        "{} {} {} {} {} {}",
        dlg::err_prefix(),
        dlg::bold("Failed to run apply on db"),
        dlg::divider(),
        dlg::red(db),
        dlg::divider(),
        e.message
      );
      exit(1);
    }
  };
}

pub async fn sync(db: &str, noninteractive: bool) {
  if !noninteractive {
    let confirmation = dlg::confirm_with_default("Press enter to confirm sync", true);
    if !confirmation {
      println!(
        "{} {} {} {}",
        dlg::warn_prefix(),
        dlg::bold("Did not run sync on db"),
        dlg::divider(),
        dlg::yellow(db)
      );
      exit(0);
    }
  }
  let sp = ProgressBar::new_spinner();
  sp.enable_steady_tick(10);
  sp.set_message("Sync in progress");
  let body = json!({
    "dbAlias": db,
  });
  let resp = post_v1("db/sync/", body).await;
  sp.finish_and_clear();
  match &resp {
    Ok(r) => {
      let plan_response: PlanResponse = serde_json::from_str(r).unwrap();
      maybe_planned_nothing(&plan_response);
      emit_plan_segment(plan_response.toCreate, &dlg::green("create").to_string());
      emit_plan_segment(plan_response.toUpdate, &dlg::yellow("update").to_string());
      emit_plan_segment(
        plan_response.toReplace,
        &dlg::magenta("replace").to_string(),
      );
      emit_plan_segment(plan_response.toDelete, &dlg::red("delete").to_string());
      println!("{} {}", dlg::success_prefix(), dlg::bold("Done"));
    }
    Err(e) => {
      eprintln!(
        "{} {} {} {} {} {}",
        dlg::err_prefix(),
        dlg::bold("Failed to run sync on db"),
        dlg::divider(),
        dlg::red(db),
        dlg::divider(),
        e.message
      );
      exit(1);
    }
  };
}

fn provide_aws_region(noninteractive: bool) -> String {
  let region_env = std::env::var("AWS_REGION");
  if region_env.is_ok() {
    return region_env.unwrap();
  } else if region_env.is_err() && noninteractive {
    println!(
      "Non interactive mode. Region needs to be defined in AWS_REGION environment variable."
    );
    std::process::exit(1);
  }
  let regions = &get_aws_regions();
  let default = regions.iter().position(|s| s == "us-east-2").unwrap_or(0);
  let selection = dlg::select_with_default("Pick AWS region to manage", regions, default);
  regions[selection].clone()
}

fn provide_aws_creds(noninteractive: bool) -> (String, String) {
  let key_env = std::env::var("AWS_ACCESS_KEY_ID");
  let secret_env = std::env::var("AWS_SECRET_ACCESS_KEY");
  if key_env.is_ok() && secret_env.is_ok() {
    return (key_env.unwrap(), secret_env.unwrap());
  } else if (key_env.is_err() || secret_env.is_err()) && noninteractive {
    println!(
      "Non interactive mode. AWS credentials needs to be defined in AWS_ACCESS_KEY_ID and AWS_SECRET_ACCESS_KEY environment variables."
    );
    std::process::exit(1);
  }
  let aws_cli_creds = get_aws_cli_creds();
  if aws_cli_creds.is_ok() {
    let all_creds = aws_cli_creds.unwrap();
    let profiles: Vec<String> = all_creds.keys().cloned().collect();
    let selection = if profiles.len() > 1 {
      dlg::select_with_default(
        "AWS CLI credentials found. Pick named profile",
        &profiles,
        0,
      )
    } else {
      println!(
        "{} {}",
        dlg::success_prefix(),
        dlg::bold("AWS CLI credentials found"),
      );
      0
    };
    let creds = all_creds.get(&profiles[selection]).unwrap();
    (
      creds.aws_access_key_id.clone(),
      creds.aws_secret_access_key.clone(),
    )
  } else {
    let access_key: String = dlg::input("AWS Access Key ID");
    let secret: String = dlg::input("AWS Secret Access Key");
    (access_key, secret)
  }
}

fn display_new_db(db_metadata: NewDbResponse) {
  let mut table = AsciiTable::default();
  table.max_width = 140;
  table.columns.insert(
    0,
    Column {
      header: "Database Server".to_string(),
      ..Column::default()
    },
  );
  table.columns.insert(
    1,
    Column {
      header: "Database Name".to_string(),
      ..Column::default()
    },
  );
  table.columns.insert(
    2,
    Column {
      header: "Username".to_string(),
      ..Column::default()
    },
  );
  table.columns.insert(
    3,
    Column {
      header: "Password".to_string(),
      ..Column::default()
    },
  );
  let server = format!("{}", dlg::bold(get_server()));
  let db = format!("{}", dlg::bold(&db_metadata.id));
  let user = format!("{}", dlg::bold(&db_metadata.user));
  let pass = format!("{}", dlg::bold(&db_metadata.password));
  let db_data = vec![vec![&server, &db, &user, &pass]];
  table.print(db_data);
  let conn_str = format!("postgres://{}:{}@{}/{}", user, pass, server, db);
  println!(
    "{} {} {} {}",
    dlg::success_prefix(),
    dlg::bold("As a PG connection string"),
    dlg::divider(),
    dlg::bold(&conn_str),
  );
  println!(
    "{} {}",
    dlg::warn_prefix(),
    dlg::bold("This is the only time we will show you these credentials, be sure to save them.",),
  );
}

pub async fn new(db_opt: Option<&str>, noninteractive: bool) -> String {
  println!(
    "{}",
    dlg::bold("Connect a cloud account to a hosted IaSQL DB...\n")
  );
  let (access_key, secret) = provide_aws_creds(noninteractive);
  let region = provide_aws_region(noninteractive);
  let db = get_or_input_db(db_opt).await;
  let sp = ProgressBar::new_spinner();
  sp.enable_steady_tick(10);
  sp.set_message("Provisioning a hosted db to manage this cloud account");
  let body = json!({
    "dbAlias": db,
    "awsRegion": region,
    "awsAccessKeyId": access_key,
    "awsSecretAccessKey": secret,
  });
  let resp = post_v1("db/new", body).await;
  sp.finish_and_clear();
  match &resp {
    Ok(res) => {
      let db_metadata: NewDbResponse = serde_json::from_str(res).unwrap();
      let alias = db_metadata.alias.clone();
      if db == alias {
        println!(
          "{} {}",
          dlg::success_prefix(),
          dlg::bold("Done provisioning hosted db")
        );
      } else {
        println!(
          "{} {} {} {}",
          dlg::success_prefix(),
          dlg::bold("Done provisioning hosted db"),
          dlg::divider(),
          dlg::green(&alias)
        );
      }
      display_new_db(db_metadata);
      alias
    }
    Err(e) => {
      eprintln!(
        "{} {} {} {} {} {}",
        dlg::err_prefix(),
        dlg::bold("Failed to add new db"),
        dlg::divider(),
        dlg::red(&db),
        dlg::divider(),
        e.message
      );
      exit(1);
    }
  }
}

fn str_from_file(file: &str) -> Result<String, Box<dyn Error>> {
  let mut source_path = std::env::current_dir().unwrap();
  source_path.push(file);
  Ok(std::fs::read_to_string(source_path)?.parse()?)
}

pub async fn import(db: &str, dump_file: &str, noninteractive: bool) {
  let dump_res = str_from_file(dump_file);
  let dump = match &dump_res {
    Ok(d) => d,
    Err(e) => {
      eprintln!(
        "{} {} {} {} {} {}",
        dlg::err_prefix(),
        dlg::bold("Failed to parse dump file"),
        dlg::divider(),
        dlg::red(&dump_file),
        dlg::divider(),
        e
      );
      exit(1);
    }
  };
  let region = provide_aws_region(noninteractive);
  let (access_key, secret) = provide_aws_creds(noninteractive);
  let sp = ProgressBar::new_spinner();
  sp.enable_steady_tick(10);
  sp.set_message("Creating a new db to manage cloud resources from a dump");
  let body = json!({
    "dbAlias": db,
    "awsRegion": region,
    "awsAccessKeyId": access_key,
    "awsSecretAccessKey": secret,
    "dump": dump
  });
  let resp = post_v1("db/import", body).await;
  sp.finish_and_clear();
  match &resp {
    Ok(res) => {
      println!("{} {}", dlg::success_prefix(), dlg::bold("Done"));
      let db_metadata: NewDbResponse = serde_json::from_str(res).unwrap();
      display_new_db(db_metadata);
    }
    Err(e) => {
      eprintln!(
        "{} {} {} {} {} {}",
        dlg::err_prefix(),
        dlg::bold("Failed to import db"),
        dlg::divider(),
        dlg::red(&db),
        dlg::divider(),
        e.message
      );
      exit(1);
    }
  };
}
