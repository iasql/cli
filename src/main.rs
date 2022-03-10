use std::env;

use clap::{crate_description, crate_name, crate_version, App, AppSettings, Arg, SubCommand};

use iasql::api::{db, module};
use iasql::auth;

extern crate iasql;

#[tokio::main]
pub async fn main() {
  // TODO add non-interactive mode support via parameters
  let app = App::new(crate_name!())
    .version(crate_version!())
    .about(crate_description!())
    .setting(AppSettings::SubcommandRequiredElseHelp)
    .subcommands(vec![
      SubCommand::with_name("login")
        .display_order(11)
        .about("Obtain and save credentials for hosted IaSQL engine")
        .arg(Arg::from_usage("--noninteractive")),
      SubCommand::with_name("new")
        .display_order(1)
        .about("Connect a hosted db to a cloud account")
        .arg(Arg::from_usage("[db]"))
        .arg(Arg::from_usage("--noninteractive")),
      SubCommand::with_name("export")
        .display_order(10)
        .about("Dump a hosted db to backup the infrastructure in the connect account")
        .arg(Arg::from_usage("[db]"))
        .arg(Arg::from_usage("[dump_file]"))
        .arg(Arg::from_usage("--noninteractive")),
      SubCommand::with_name("remove")
        .display_order(2)
        .about("Remove a hosted db to stop managing the connected cloud account")
        .visible_alias("rm")
        .arg(Arg::from_usage("[db]"))
        .arg(Arg::from_usage("--noninteractive")),
      SubCommand::with_name("apply")
        .display_order(4)
        .about("Create, delete or update the cloud resources in a hosted db")
        .arg(Arg::from_usage("[db]"))
        .arg(Arg::from_usage("--noninteractive")),
      SubCommand::with_name("plan")
        .display_order(5)
        .about("Display a preview of the resources in a db to be modified on the next `apply`")
        .arg(Arg::from_usage("[db]"))
        .arg(Arg::from_usage("--noninteractive")),
      SubCommand::with_name("sync")
        .display_order(6)
        .about("Synchronize a hosted db with the current state of the cloud account")
        .arg(Arg::from_usage("[db]"))
        .arg(Arg::from_usage("--noninteractive")),
      SubCommand::with_name("install")
        .display_order(7)
        .about("Install mods in a given hosted db")
        .arg(Arg::from_usage("--db=[DB]"))
        .arg(Arg::with_name("modules").min_values(1))
        .arg(Arg::from_usage("--noninteractive")),
      SubCommand::with_name("uninstall")
        .display_order(8)
        .about("Uninstall mods from a given hosted db")
        .arg(Arg::from_usage("--db=[DB]"))
        .arg(Arg::with_name("modules").min_values(1))
        .arg(Arg::from_usage("--noninteractive")),
      SubCommand::with_name("logout")
        .display_order(12)
        .about("Remove locally-stored credentials for the hosted IaSQL engine")
        .arg(Arg::from_usage("--noninteractive")),
      SubCommand::with_name("dbs")
        .display_order(3)
        .alias("databases")
        .about("List all hosted dbs")
        .arg(Arg::from_usage("--noninteractive")),
      SubCommand::with_name("mods")
        .display_order(9)
        .alias("modules")
        .about("List all modules or list the modules installed in a given hosted db")
        .arg(Arg::from_usage("[db]"))
        .arg(Arg::from_usage("--noninteractive")),
    ]);

  match app.get_matches().subcommand() {
    ("login", Some(s_matches)) => {
      let noninteractive = s_matches.is_present("noninteractive");
      auth::login(true, noninteractive).await;
    }
    ("logout", Some(s_matches)) => {
      let noninteractive = s_matches.is_present("noninteractive");
      auth::logout(noninteractive);
    }
    ("new", Some(s_matches)) => {
      let noninteractive = s_matches.is_present("noninteractive");
      auth::login(false, noninteractive).await;
      let db = db::get_or_input_db(s_matches.value_of("db")).await;
      db::new(&db, noninteractive).await;
      if !noninteractive {
        let modules = module::mods_to_install(&db, None).await;
        module::install(&db, modules, noninteractive).await;
      }
    }
    ("import", Some(s_matches)) => {
      let noninteractive = s_matches.is_present("noninteractive");
      auth::login(false, noninteractive).await;
      let db = db::get_or_input_db(s_matches.value_of("db")).await;
      let dump_file = db::get_or_input_arg(s_matches.value_of("dump_file"), "Dump file");
      db::import(&db, &dump_file, noninteractive).await;
    }
    ("export", Some(s_matches)) => {
      let noninteractive = s_matches.is_present("noninteractive");
      auth::login(false, noninteractive).await;
      let db = db::get_or_select_db(s_matches.value_of("db")).await;
      let dump_file = db::get_or_input_arg(s_matches.value_of("dump_file"), "Dump file");
      db::export(&db, dump_file).await;
    }
    ("remove", Some(s_matches)) => {
      let noninteractive = s_matches.is_present("noninteractive");
      auth::login(false, noninteractive).await;
      let db = db::get_or_select_db(s_matches.value_of("db")).await;
      db::remove(&db, noninteractive).await;
    }
    ("apply", Some(s_matches)) => {
      let noninteractive = s_matches.is_present("noninteractive");
      auth::login(false, noninteractive).await;
      let db = db::get_or_select_db(s_matches.value_of("db")).await;
      db::apply(&db, noninteractive).await;
    }
    ("plan", Some(s_matches)) => {
      let noninteractive = s_matches.is_present("noninteractive");
      auth::login(false, noninteractive).await;
      let db = db::get_or_select_db(s_matches.value_of("db")).await;
      db::plan(&db, noninteractive).await;
    }
    ("sync", Some(s_matches)) => {
      let noninteractive = s_matches.is_present("noninteractive");
      auth::login(false, noninteractive).await;
      let db = db::get_or_select_db(s_matches.value_of("db")).await;
      db::sync(&db, noninteractive).await;
    }
    ("dbs", Some(s_matches)) => {
      let noninteractive = s_matches.is_present("noninteractive");
      auth::login(false, noninteractive).await;
      db::list().await;
    }
    ("mods", Some(s_matches)) => {
      let noninteractive = s_matches.is_present("noninteractive");
      auth::login(false, noninteractive).await;
      module::list(s_matches.value_of("db")).await;
    }
    ("install", Some(s_matches)) => {
      let noninteractive = s_matches.is_present("noninteractive");
      auth::login(false, noninteractive).await;
      let db = db::get_or_select_db(s_matches.value_of("db")).await;
      let modules = module::mods_to_install(&db, s_matches.values_of_lossy("modules")).await;
      module::install(&db, modules, noninteractive).await;
    }
    ("uninstall", Some(s_matches)) => {
      let noninteractive = s_matches.is_present("noninteractive");
      auth::login(false, noninteractive).await;
      let db = db::get_or_select_db(s_matches.value_of("db")).await;
      let modules = module::mods_to_remove(&db, s_matches.values_of_lossy("modules")).await;
      module::uninstall(&db, modules, noninteractive).await;
    }
    // rely on AppSettings::SubcommandRequiredElseHelp
    _ => {}
  }
}
