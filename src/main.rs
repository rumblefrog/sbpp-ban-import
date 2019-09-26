use structopt::StructOpt;
use steamid_ng::SteamID;

#[macro_use]
extern crate mysql;

#[derive(Debug, StructOpt)]
struct CliArg {
    host: String,

    user_name: String,

    password: String,

    port: u16,

    database: String,

    table_prefix: String,

    #[structopt(parse(from_os_str))]
    file_path: std::path::PathBuf,
}

fn main() {
    let args = CliArg::from_args();

    let content = std::fs::read_to_string(&args.file_path).unwrap();

    let mut builder = mysql::OptsBuilder::new();

    builder
        .ip_or_hostname(Some(args.host))
        .db_name(Some(args.database))
        .tcp_port(args.port)
        .user(Some(args.user_name))
        .pass(Some(args.password));

    let pool = mysql::Pool::new(builder).unwrap();

    let table = format!("{}_bans", args.table_prefix);

    let mut ban_count: u64 = 0;

    for line in content.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();

        if parts[0] == "addip" {
            let mut ip_iter = pool.prep_exec(format!("SELECT `ip` FROM {} WHERE ip = :ip AND RemoveType IS NULL", table), params!{"ip" => parts[2]}).unwrap().into_iter();

            if let Some(_v) = ip_iter.next() {
                continue;
            }

            pool.prep_exec(
                format!(
                    "INSERT INTO {} (`created`, `authid`, `ip`, `name`, `ends`, `length`, `reason`, `type`) VALUES (UNIX_TIMESTAMP(), '', :ip, 'Imported Ban', (UNIX_TIMESTAMP() + 0), 0, 'banned_ip.cfg import', 1)",
                    table
                ),
                params!{"ip" => parts[2]}
            ).unwrap();

            ban_count = ban_count + 1;
        }
        else if parts[0] == "banid" {
            let steam = SteamID::from_steam3(parts[2]).unwrap().steam2();

            let mut id_iter = pool.prep_exec(format!("SELECT `authid` FROM {} WHERE authid = :authid AND RemoveType IS NULL", table), params!{"authid" => &steam}).unwrap().into_iter();

            if let Some(_v) = id_iter.next() {
                continue;
            }

            let mut stmt = pool.prepare(
                format!(
                    "INSERT INTO {} (`created`, `authid`, `ip`, `name`, `ends`, `length`, `reason`, `type`) VALUES (UNIX_TIMESTAMP(), :authid, '', :name, (UNIX_TIMESTAMP() + 0), 0, 'banned_user.cfg import', 0)",
                    table,
                )
            ).unwrap();

            stmt.execute(params!{
                "authid" => &steam,
                "name" => "Imported Ban",
            }).unwrap();

            ban_count = ban_count + 1;
        }
    }

    println!("Imported {} bans", ban_count);
}
