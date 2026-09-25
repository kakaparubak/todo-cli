use clap::{Args, Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "todo", version, about = "A simple CLI todo manager")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    Add(AddArgs),
    List(ListArgs),
    Done { id: i64 },
    Undone { id: i64 },
    Edit(EditArgs),
    Delete { id: i64 },
    Restore { id: i64 },
}

#[derive(Args, Debug)]
pub struct AddArgs {
    pub title: String,
    #[arg(short, long, default_value = "medium")]
    pub priority: String,
    #[arg(short, long)]
    pub due: Option<String>,
    #[arg(short, long, value_delimiter = ',')]
    pub tags: Vec<String>,
}

#[derive(Args, Debug)]
pub struct ListArgs {
    #[arg(long)]
    pub all: bool,
    #[arg(long)]
    pub tag: Option<String>,
    #[arg(long)]
    pub priority: Option<String>,
    #[arg(long)]
    pub search: Option<String>,
    #[arg(long)]
    pub trash: bool,
}

#[derive(Args, Debug)]
pub struct EditArgs {
    pub id: i64,
    #[arg(long)]
    pub title: Option<String>,
    #[arg(long)]
    pub priority: Option<String>,
    #[arg(long)]
    pub due: Option<String>,
    #[arg(long, value_delimiter = ',')]
    pub tags: Option<Vec<String>>,
}