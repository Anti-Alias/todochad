use std::fmt;
use std::{env, fs};
use std::path::{PathBuf, Path};
use clap::{command, Parser, Subcommand};
use thiserror::Error;
use tabled::{Table, Tabled};
use tdc::{Graph, GraphError, Task, TaskId, TaskStatus};

const APP_NAME: &str        = "tdc";
const GRAPH_FILE_NAME: &str = "graph.ron";

#[derive(Parser, Debug)]
#[command(name="tdc")]
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
        #[clap(help="Name of the task")]
        task_name: String,
    },
    #[command(name="rm", about="Remove a task")]
    Remove { 
        #[clap(help="Id of the task")]
        task_id: TaskId,
    }, 
    #[command(name="ls", about="List all tasks")]
    List,
    #[command(name="sel", about="Selects a task, adding it to the todo list")]
    Select { 
        #[clap(short, long, help="Selects all tasks if set")]
        all: bool,
        #[clap(help="Id of the task(s)")]
        task_ids: Vec<TaskId>,
    },
    #[command(name="desel", about="Deselects a task, removing it from the todo list")]
    Deselect { 
        #[clap(help="Id of the task(s)")]
        task_ids: Vec<TaskId>,
        #[clap(short, long, help="Deselects all tasks if set")]
        all: bool,
    },
    #[command(name="finish", about="Finishes a task")]
    Finish { 
        #[clap(help="Id of the task")]
        task_id: TaskId,
    },
    #[command(name="todo", about="Produces a todo list of tasks using currently selected tasks")]
    Todo {
        #[clap(long, short, help="Show all tasks on todo list, including those with unmet dependencies")]
        all: bool,
    },
    #[command(name="destroy", about="Destroys all tasks")]
    Destroy,
    #[command(name="depadd", about="Add dependencies to a task")]
    DepAdd {
        #[clap(help="Id of task receiving dependencies")]
        task_id: TaskId,
        #[clap(required=true, help="Ids of tasks that will added as dependencies")]
        child_ids: Vec<TaskId>,
    },
    #[command(name="deprm", about="Remove dependencies from a task")]
    DepRemove {
        #[clap(help="Id of task removing dependencies")]
        task_id: TaskId,
        #[clap(required=true, help="Ids of tasks that will be removed as dependencies")]
        child_ids: Vec<TaskId>,
    },
    #[command(name="depclear", about="Clear dependencies of a task")]
    DepClear {
        #[clap(help="Id of task clearing dependencies")]
        task_id: TaskId,
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
        Command::Select { task_ids, all } => {
            let mut graph = load_graph(&graph_path)?;
            if all {
                graph.select_all();
            }
            else {
                for task_id in task_ids {
                    graph.set_status(task_id, TaskStatus::Selected)?;
                }
            }
            save_graph(&graph_path, &graph)?;
        },
        Command::Deselect { task_ids, all } => {
            let mut graph = load_graph(&graph_path)?;
            if all {
                graph.deselect_all();
            }
            else {
                for task_id in task_ids {
                    graph.set_status(task_id, TaskStatus::Deselected)?;
                }
            }
            save_graph(&graph_path, &graph)?;
        },
        Command::Finish { task_id } => {
            let mut graph = load_graph(&graph_path)?;
            graph.set_status(task_id, TaskStatus::Finished)?;
            save_graph(&graph_path, &graph)?;
        },
        Command::Todo { all } => {
            let graph = load_graph(&graph_path)?;
            let tasks = graph.traverse_selected();
            let task_rows = map_tasks_to_rows(tasks, &graph);
            let mut task_rows: Vec<TaskRow> =  task_rows
                .filter(|task| all || task.doable)
                .collect();
            task_rows.sort_by_key(|task_row| (task_row.doable, task_row.selected));
            print_task_rows(&task_rows);
        },
        Command::List => {
            let graph = load_graph(&graph_path)?;
            let task_rows = map_tasks_to_rows(graph.iter(), &graph);
            let mut task_rows: Vec<TaskRow> = task_rows.collect();
            task_rows.sort_by_key(|task_row| (task_row.selected, task_row.doable, !task_row.finished));
            print_task_rows(&task_rows);
        },
        Command::Destroy => {
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
        Command::DepClear { task_id } => {
            let mut graph = load_graph(&graph_path)?;
            graph.clear_children(task_id)?;
            save_graph(&graph_path, &graph)?;
        },
    }
    Ok(())
}

fn print_task_rows(task_rows: &[TaskRow]) {
    let task_table = Table::new(task_rows);
    println!("{task_table}");
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
    selected: bool,
    finished: bool,
    doable: bool,
    dependencies: Dependencies<'a>, 
}

impl<'a> TaskRow<'a> {
    fn new(id: TaskId, task: &'a Task, doable: bool) -> Self {
        let name = &task.name;
        let dependencies = Dependencies(task.children());
        let (selected, finished) = match task.status() {
            TaskStatus::Selected => (true, false),
            TaskStatus::Deselected => (false, false),
            TaskStatus::Finished => (false, true),
        };
        Self { id, name, selected, finished, doable, dependencies }
    }
}

fn map_tasks_to_rows<'g, T>(tasks: T, graph: &'g Graph) -> impl Iterator<Item=TaskRow<'g>> 
where 
    T: IntoIterator<Item=(TaskId, &'g Task)>,
{
    tasks
        .into_iter()
        .map(|(task_id, task)| {
            let doable = graph.has_met_dependencies(task_id).unwrap();
            TaskRow::new(task_id, task, doable)
        })
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
