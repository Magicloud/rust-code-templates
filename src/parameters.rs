use clap::*;

#[derive(Parser, Clone, Debug)]
pub struct PGParams {
    #[arg(long, env = "PGHOST")]
    pub pg_host: String,
    #[arg(long, env = "PGPORT", default_value = "5432")]
    pub pg_port: u16,
    #[arg(long, env = "PGDATABASE")]
    pub pg_database: String,
    #[arg(long, env = "PGUSERNAME")]
    pub pg_username: String,
    #[arg(long, env = "PGPASSWORD")]
    pub pg_password: String,
}

#[derive(Parser, Clone, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Params {
    #[command(flatten)]
    pub pg_params: PGParams,
    #[arg(short, long, default_value = "memcache://localhost:11211")]
    pub memcached_address: String,
    #[arg(short, long, default_value = "localhost:3000")]
    pub listen_address: String,
}
