use std::env;

use clap::{crate_name, crate_version, App, AppSettings, Arg, SubCommand};

use iasql::api::{db, module};
use iasql::auth;

extern crate iasql;

#[tokio::main]
pub async fn main() {
  // TODO add non-interactive mode support via parameters
  let app = App::new(crate_name!())
    .version(crate_version!())
    .setting(AppSettings::SubcommandRequiredElseHelp)
    .subcommands(vec![
      SubCommand::with_name("login"),
      SubCommand::with_name("db")
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .alias("database")
        .subcommands(vec![
          SubCommand::with_name("list").visible_alias("ls"),
          SubCommand::with_name("add"),
          SubCommand::with_name("remove").visible_alias("rm"),
          SubCommand::with_name("apply"),
        ]),
      SubCommand::with_name("mod")
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .alias("module")
        .subcommands(vec![
          SubCommand::with_name("list")
            .visible_alias("ls")
            .setting(AppSettings::SubcommandRequiredElseHelp)
            .subcommands(vec![
              SubCommand::with_name("installed").arg(Arg::from_usage("[db]")),
              SubCommand::with_name("all"),
            ]),
          SubCommand::with_name("install")
            .alias("add")
            .arg(Arg::from_usage("--db=[DB]"))
            .arg(Arg::with_name("modules").min_values(1)),
          SubCommand::with_name("remove")
            .visible_alias("rm")
            .arg(Arg::from_usage("--db=[DB]"))
            .arg(Arg::with_name("modules").min_values(1)),
        ]),
      SubCommand::with_name("logout"),
    ]);

  let matches = app.get_matches();
  match matches.subcommand() {
    ("db", Some(s_matches)) => {
      auth::login(false).await;
      match s_matches.subcommand() {
        ("list", _) => db::list().await,
        ("add", _) => db::add().await,
        ("remove", _) => db::remove().await,
        ("apply", _) => db::apply().await,
        // rely on AppSettings::SubcommandRequiredElseHelp
        _ => {}
      };
    }
    ("mod", Some(s_matches)) => {
      auth::login(false).await;
      match s_matches.subcommand() {
        ("list", Some(s_s_matches)) => {
          match s_s_matches.subcommand() {
            ("installed", Some(s_s_s_matches)) => {
              let db = module::get_or_select_db(s_s_s_matches.value_of("db")).await;
              module::list(Some(&db)).await;
            }
            ("all", _) => module::list(None).await,
            // rely on AppSettings::SubcommandRequiredElseHelp
            _ => {}
          };
        }
        ("install", Some(s_s_matches)) => {
          let db = module::get_or_select_db(s_s_matches.value_of("db")).await;
          let modules = module::mods_to_install(&db, s_s_matches.values_of_lossy("modules")).await;
          module::install(&db, modules).await;
        }
        ("remove", Some(s_s_matches)) => {
          let db = module::get_or_select_db(s_s_matches.value_of("db")).await;
          let modules = module::mods_to_rm(&db, s_s_matches.values_of_lossy("modules")).await;
          module::remove(&db, modules).await;
        }
        // rely on AppSettings::SubcommandRequiredElseHelp
        _ => {}
      };
    }
    ("login", _) => {
      auth::login(true).await;
    }
    ("logout", _) => {
      auth::logout();
    }
    // rely on AppSettings::SubcommandRequiredElseHelp
    _ => {}
  }
}