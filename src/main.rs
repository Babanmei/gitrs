#[macro_use]
extern crate anyhow;
#[cfg(feature = "yaml")]
#[macro_use]
extern crate clap;

use anyhow::Result;

use gitrs::rep::Repository;

pub fn cli() -> Result<()> {
    let yaml = clap::load_yaml!("cli.yml");
    let matches = clap::App::from(yaml).get_matches();
    let opt = matches.subcommand().ok_or_else(|| anyhow!("cli"))?;
    match opt {
        ("init", _login_matchs) => {
            println!("init cmd")
        }

        ("add", files) => {
            let rep = Repository::new()?;
            let file = files.value_of("files").ok_or_else(|| anyhow!("please take filename"))?;
            rep.add(file)?;
        }

        ("status", _s_matchs) => {
            println!("status")
        }

        ("commit", c_matchs) => {
            let msg = c_matchs.value_of("msg").ok_or_else(|| anyhow!("please take commit msg"))?;
            let rep = Repository::new()?;
            rep.commit(msg)?;
        }

        ("checkout", c_matchs) => {
            let new = c_matchs.occurrences_of("branch") == 1;
            let name = c_matchs.value_of("name").ok_or_else(|| anyhow!("please take branch name"))?;

            let rep = Repository::new()?;
            rep.checkout(name, new)?;
        }

        ("branch", b_matchs) => {
            let _new = b_matchs.occurrences_of("branch") == 1;
            let rep = Repository::new()?;
            rep.list_branch_names()?;
        }
        _ => {}
    }
    Ok(())
}

fn main() {
    let res = cli();
    if res.is_err() {
        let err = res.unwrap_err().to_string();
        println!("{}", err);
    }
}
