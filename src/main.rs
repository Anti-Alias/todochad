use std::fmt;
use std::{env, fs};
use std::path::{PathBuf, Path};
use clap::{command, Parser, Subcommand};
use thiserror::Error;
use tabled::{Table, Tabled};
use tod::{Graph, GraphError, Task, TaskId};

const APP_NAME: &str        = "tod";
const GRAPH_FILE_NAME: &str = "graph.ron";

#[derive(Parser, Debug)]
#[command(name="tod")]
#[command(version="0.1")]
#[command(about="A todo list generator using a dependency graph")]
struct Cli {
    #[command(subcommand)]
    command: Command,  
}

#[derive(Subcommand, Debug)]
enum Command { 
    #[command(name="add", about="Add a task")]
    Add { 
        task_name: String,
    },
    #[command(name="rm", about="Remove a task")]
    Remove { 
        task_id: TaskId,
    }, 
    #[command(name="ls", about="List tasks with table decorations")]
    List {
        #[clap(long, short, help="Strips away table decorations")]
        simple: bool,
    },
    #[command(name="clear", about="Clear all tasks")]
    Clear,
    #[command(name="depadd", about="Adds child tasks as dependencies of another task")]
    DepAdd {
        #[clap(help="Task that will receive child tasks")]
        task_id: TaskId,
        #[clap(required=true, help="Child task(s) to add")]
        child_ids: Vec<TaskId>,
    },
    #[command(name="deprm", about="Removes child tasks as dependencies from another task")]
    DepRemove {
        #[clap(help="Task that will remove child tasks")]
        task_id: TaskId,
        #[clap(required=true, help="Child task(s) to remove")]
        child_ids: Vec<TaskId>,
    },
}

fn main() {
    if let Err(err) = run() {
        eprintln!("Error: {err}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();
    run_command(cli.command)?;
    Ok(())
}

/// Runs a command on a graph.
/// Returns true if the graph was edited.
fn run_command(command: Command) -> Result<()> {
    let graph_path = graph_path()?;
    match command {
        Command::Add { task_name } => {
            let mut graph = load_graph(&graph_path)?;
            let task_id = graph.insert(Task::new(task_name));
            save_graph(&graph_path, &graph)?;
            println!("{task_id}");
        },
        Command::Remove { task_id } => {
            let mut graph = load_graph(&graph_path)?;
            graph.remove(task_id).ok_or(GraphError::TaskNotFound)?;
            save_graph(&graph_path, &graph)?;
        },
        Command::List { simple } => {
            let graph = load_graph(&graph_path)?;
            if !simple {
                let task_rows = graph.iter().map(|(task_id, task)| TaskRow::new(task_id, task));
                let task_table = Table::new(task_rows);
                println!("{task_table}");
            }
            else {
                for (task_id, task) in graph.iter() {
                    let task_row = TaskRow::new(task_id, task);
                    task_row.print();
                }
            }
        },
        Command::Clear => {
            match fs::exists(&graph_path) { 
                Err(_) => return Err(AppError::GraphReadError),
                Ok(false) => {},
                Ok(true) => {
                    std::fs::remove_file(graph_path).map_err(|_| AppError::GraphDeleteError)?;
                },
            }
        },
        Command::DepAdd { task_id, child_ids } => {
            let mut graph = load_graph(&graph_path)?;
            for child_id in child_ids {
                graph.insert_child(task_id, child_id)?;
            }
            save_graph(&graph_path, &graph)?;
        },
        Command::DepRemove { task_id, child_ids } => {
            let mut graph = load_graph(&graph_path)?;
            for child_id in child_ids {
                graph.remove_child(task_id, child_id)?;
            }
            save_graph(&graph_path, &graph)?;
        },
    }
    Ok(())
}

fn load_graph(graph_path: &Path) -> Result<Graph> {
    match fs::exists(graph_path) { 
        Err(_) => Err(AppError::GraphReadError),
        Ok(false) => Ok(Graph::new()),
        Ok(true) => {
            let graph_str = fs::read_to_string(graph_path).map_err(|_| AppError::GraphReadError)?;
            let graph = ron::from_str(&graph_str).map_err(|_| AppError::GraphParseError)?;
            Ok(graph)
        },
    }
}

fn save_graph(graph_path: &Path, graph: &Graph) -> Result<()> {
    let graph_str = ron::to_string(graph).expect("Failed to serialize graph");
    fs::write(graph_path, graph_str).map_err(|_| AppError::GraphWriteError)?;
    Ok(())
}

/// Determines path of graph file.
/// Creates directory structure along the way if it does not exist.
fn graph_path() -> Result<PathBuf> {
    let home = env::var("HOME").map_err(|_| AppError::HomeDirError)?;
    let graph_path = format!("{home}/.local/share/{APP_NAME}/{GRAPH_FILE_NAME}");
    let graph_path = PathBuf::from(graph_path);
    if let Some(graph_dir) = graph_path.parent() {
        let res = fs::create_dir_all(graph_dir);
        if res.is_err() {
            return Err(AppError::HomeDirError);
        }
    }
    Ok(graph_path)
}

/// Printable task record
#[derive(Tabled)]
struct TaskRow<'a> {
    id: TaskId,
    name: &'a str,
    dependencies: Dependencies<'a>, 
}

impl<'a> TaskRow<'a> {
    fn new(id: TaskId, task: &'a Task) -> Self {
        Self {
            id,
            name: &task.name,
            dependencies: Dependencies(task.children()),
        }
    }

    fn print(&self) {
        println!("{} \"{}\" {}", self.id, self.name, self.dependencies);
    }
}

/// Printable list of a task's dependencies
struct Dependencies<'a>(&'a [TaskId]);
impl fmt::Display for Dependencies<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.0.len() {
            1 => write!(f, "{}", self.0[0])?,
            2.. => {
                write!(f, "{}", self.0[0])?;
                for task_id in &self.0[1..] {
                    write!(f, ",{task_id}")?;
                }
            },
            _ => {},
        }
        Ok(())
    }
}


#[derive(Error, Debug)]
pub enum AppError {
    #[error("Failed to get home directory")]
    HomeDirError,
    #[error("Failed to read graph file")]
    GraphReadError,
    #[error("Failed to parse graph file")]
    GraphParseError,
    #[error("Failed to write graph file")]
    GraphWriteError,
    #[error("Failed delete graph file")]
    GraphDeleteError,
    #[error(transparent)]
    GraphError(#[from] GraphError),
}

type Result<T> = std::result::Result<T, AppError>;
