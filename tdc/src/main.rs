use std::fmt;
use clap::{command, Parser, Subcommand};
use thiserror::Error;
use tabled::{Table, Tabled};
use tdc::{Config, ConfigError, Graph, GraphError, Task, TaskId, TaskOrder};
use glob::{Pattern, PatternError};

const INDENT: &str = "    ";

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
        #[clap(short, long, help="Removes all tasks if set")]
        all: bool,
        #[clap(help="Id of the task(s)")]
        task_ids: Vec<TaskId>,
    }, 
    #[command(name="rename", about="Rename a task")]
    Rename { 
        #[clap(help="Id of the task")]
        task_id: TaskId,
        #[clap(help="New name of the task")]
        name: String,
    }, 
    #[command(name="ls", about="List all tasks")]
    List,
    #[command(name="tree", about="Prints a tree view of one or more tasks and their dependencies")]
    Tree {
        #[clap(help="Id of the task(s)")]
        task_ids: Vec<TaskId>,
    },
    #[command(name="find", about="Find tasks whose name contains the pattern provided")]
    Find {
        #[clap(help="Pattern to search for")]
        pattern: String,
        #[clap(short, long, help="Treats pattern as a glob pattern if set")]
        glob: bool,
        #[clap(short, long, help="Matching will be case sensitive")]
        case_sensitive: bool,
    },
    #[command(name="sel", about="Select a task, adding it to the todo list")]
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
    #[command(name="todo", about="Show the todo list of tasks using currently selected tasks")]
    Todo {
        #[clap(long, short, help="Shows all tasks on todo list, including those with dependencies")]
        all: bool,
    },
    #[command(name="depadd", about="Add dependencies to a task")]
    DepAdd {
        #[clap(help="Id of task receiving dependencies")]
        task_id: TaskId,
        #[clap(required=true, help="Ids of tasks that will added as dependencies")]
        dependency_ids: Vec<TaskId>,
    },
    #[command(name="deprm", about="Remove dependencies from a task")]
    DepRemove {
        #[clap(help="Id of task removing dependencies")]
        task_id: TaskId,
        #[clap(required=true, help="Ids of tasks that will be removed as dependencies")]
        dependency_ids: Vec<TaskId>,
    },
    #[command(name="depclear", about="Clear dependencies of a task")]
    DepClear {
        #[clap(help="Id of task clearing dependencies")]
        task_id: TaskId,
    },
    #[command(name="order", about="Set the order of a task using an integer. If not set, order is cleared.")]
    Order {
        #[clap(help="Id of task being ordered")]
        task_id: TaskId,
        order: Option<i32>,
    },
    #[command(name="tags", about="Lists all tags across all tasks.")]
    Tags,    
    #[command(name="tagadd", about="Add searchable tags to a task.")]
    TagAdd {
        #[clap(help="Task to add tags to")]
        task_id: TaskId,
        tags: Vec<String>,
    },
    #[command(name="tagrm", about="Removes tags from a task.")]
    TagRemove {
        #[clap(help="Task to add a tag to")]
        task_id: TaskId,
        tags: Vec<String>,
    },
    #[command(name="tagfind", about="Finds a task that has all of the tags specified.")]
    TagFind {
        tags: Vec<String>,
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
    let config = Config::load()?;
    match command {
        Command::Add { task_name } => {
            let mut graph = Graph::load(&config)?;
            let task_id = graph.insert(Task::new(task_name));
            graph.save(&config)?;
            println!("{task_id}");
        },
        Command::Remove { task_ids, all } => {
            let mut graph = Graph::load(&config)?;
            if all {
                graph.clear();
            }
            else if !task_ids.is_empty() {
                for task_id in task_ids {
                    graph.remove(task_id).ok_or(GraphError::TaskNotFound)?;
                }
            }
            else {
                return Err(AppError::MissingTaskListOrAllFlag);
            }
            graph.save(&config)?;
        },
        Command::Rename { task_id, name } => {
            let mut graph = Graph::load(&config)?;
            let task = graph.get_mut(task_id).ok_or(GraphError::TaskNotFound)?;
            task.name = name;
            graph.save(&config)?;
        },
        Command::Select { task_ids, all } => {
            let mut graph = Graph::load(&config)?;
            if all {
                graph.set_selected_all(true);
            }
            else if !task_ids.is_empty() {
                for task_id in task_ids {
                    graph.set_selected(task_id, true)?;
                }
            }
            else {
                return Err(AppError::MissingTaskListOrAllFlag);
            }
            graph.save(&config)?;
        },
        Command::Deselect { task_ids, all } => {
            let mut graph = Graph::load(&config)?;
            if all {
                graph.set_selected_all(false);
            }
            else if !task_ids.is_empty() {
                for task_id in task_ids {
                    graph.set_selected(task_id, false)?;
                }
            }
            else {
                return Err(AppError::MissingTaskListOrAllFlag);
            }
            graph.save(&config)?;
        },
        Command::Todo { all } => {
            let graph = Graph::load(&config)?;
            let tasks = graph.traverse_selected();
            let mut task_rows: Vec<TaskRow> = tasks
                .into_iter()
                .map(|(task_id, task)| TaskRow::new(task_id, task))
                .filter(|task| all || task.doable())
                .collect();
            task_rows.sort_by_key(|task_row| (!task_row.doable(), task_row.order));
            let task_table = Table::new(task_rows);
            println!("{task_table}");
        },
        Command::List => {
            let graph = Graph::load(&config)?;
            let mut task_rows: Vec<TaskRow> = graph.iter() 
                .map(|(task_id, task)| TaskRow::new(task_id, task))
                .collect();
            task_rows.sort_by_key(|task_row| !task_row.selected);
            let task_table = Table::new(task_rows);
            println!("{task_table}");
        },
        Command::Tree { task_ids } => {
            let graph = Graph::load(&config)?;
            for task_id in task_ids {
                let task = graph.get(task_id).ok_or(GraphError::TaskNotFound)?;
                let task_tree = TaskTree { task_id, task, graph: &graph };
                print!("{task_tree}");
            }
        },
        Command::Find { pattern, glob, case_sensitive } => {
            let graph = Graph::load(&config)?;
            let mut task_rows: Vec<TaskRow> = match (glob, case_sensitive) {
                (false, false) => {
                    let pattern = pattern.to_uppercase();
                    graph.iter() 
                        .filter(|(_, task)| task.name.to_uppercase().contains(&pattern))
                        .map(|(task_id, task)| TaskRow::new(task_id, task))
                        .collect()
                },
                (false, true) => {
                    graph.iter() 
                        .filter(|(_, task)| task.name.contains(&pattern))
                        .map(|(task_id, task)| TaskRow::new(task_id, task))
                        .collect()
                },
                (true, false) => {
                    let pattern = Pattern::new(&pattern.to_uppercase())?;
                    graph.iter() 
                        .filter(|(_, task)| pattern.matches(&task.name.to_uppercase()))
                        .map(|(task_id, task)| TaskRow::new(task_id, task))
                        .collect()
                },
                (true, true) => {
                    let pattern = Pattern::new(&pattern)?;
                    graph.iter() 
                        .filter(|(_, task)| pattern.matches(&task.name))
                        .map(|(task_id, task)| TaskRow::new(task_id, task))
                        .collect()
                },
            };
            task_rows.sort_by_key(|task_row| !task_row.selected);
            let task_table = Table::new(task_rows);
            println!("{task_table}");
        },
        Command::DepAdd { task_id, dependency_ids } => {
            let mut graph = Graph::load(&config)?;
            for dependency_id in dependency_ids {
                graph.insert_dependency(task_id, dependency_id)?;
            }
            graph.save(&config)?;
        },
        Command::DepRemove { task_id, dependency_ids } => {
            let mut graph = Graph::load(&config)?;
            for dependency_id in dependency_ids {
                graph.remove_dependency(task_id, dependency_id)?;
            }
            graph.save(&config)?;
        },
        Command::DepClear { task_id } => {
            let mut graph = Graph::load(&config)?;
            graph.clear_dependencies(task_id)?;
            graph.save(&config)?;
        },
        Command::Order { task_id, order } => {
            let mut graph = Graph::load(&config)?;
            let task = graph.get_mut(task_id).ok_or(GraphError::TaskNotFound)?;
            let order = match order { 
                Some(order) => TaskOrder::Order(order),
                None => TaskOrder::Last,
            };
            task.order = order;
            graph.save(&config)?;
        },
        Command::Tags => {
            let graph = Graph::load(&config)?;
            for task in graph.tags() {
                println!("{task}");
            }
        },
        Command::TagAdd { task_id, tags } => {
            let mut graph = Graph::load(&config)?;
            let mut modified = false;
            let task = graph.get_mut(task_id).ok_or(GraphError::TaskNotFound)?;
            for tag in tags {
                if task.add_tag(tag) {
                    modified = true;
                }
            }
            if modified {
                graph.save(&config)?;
            }
        },
        Command::TagRemove { task_id, tags } => {
            let mut graph = Graph::load(&config)?;
            let mut modified = false;
            let task = graph.get_mut(task_id).ok_or(GraphError::TaskNotFound)?;
            for tag in tags {
                if task.remove_tag(&tag) {
                    modified = true;
                }
            }
            if modified {
                graph.save(&config)?;
            }
        },
        Command::TagFind { tags } => {
            let graph = Graph::load(&config)?;
            let task_rows: Vec<TaskRow> = graph.iter_with_tags(&tags)
                .map(|(task_id, task)| TaskRow::new(task_id, task))
                .collect();
            let task_table = Table::new(task_rows);
            println!("{task_table}");
        },
    }
    Ok(())
}

struct TaskTree<'a> {
    task_id: TaskId,
    task: &'a Task,
    graph: &'a Graph,
}
impl fmt::Display for TaskTree<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        print_task_tree(self.task_id, self.task, self.graph, 0, f)
    }
}

fn print_task_tree(
    task_id: TaskId,
    task: &Task,
    graph: &Graph,
    indentation: u32,
    f: &mut fmt::Formatter,
) -> fmt::Result {
    for _ in 0..indentation {
        write!(f, "{}", INDENT)?;
    }
    writeln!(f, "{}) {}", task_id, task.name)?;
    for dep_id in task.dependencies().iter().copied() {
        let dep_task = graph.get(dep_id).unwrap();
        print_task_tree(dep_id, dep_task, graph, indentation + 1, f)?;
    }
    Ok(())
}


/// Printable task record
#[derive(Tabled)]
struct TaskRow<'a> {
    id: TaskId,
    name: &'a str,
    tags: Tags<'a>,
    selected: bool,
    order: TaskOrder,
    dependencies: Dependencies<'a>, 
}

impl<'a> TaskRow<'a> {
    fn new(id: TaskId, task: &'a Task) -> Self {
        Self {
            id, 
            name: &task.name, 
            tags: Tags(task.tags()),
            selected: task.selected,
            order: task.order, 
            dependencies: Dependencies(task.dependencies()),
        }
    }
     fn doable(&self) -> bool {
         self.dependencies.0.is_empty()
     }
}

/// Printable list of a task's dependencies
struct Dependencies<'a>(&'a [TaskId]);
impl fmt::Display for Dependencies<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.0.len() {
            0 => {},
            1 => write!(f, "{}", self.0[0])?,
            2.. => {
                write!(f, "{}", self.0[0])?;
                for task_id in &self.0[1..] {
                    write!(f, ",{task_id}")?;
                }
            },
        }
        Ok(())
    }
}

/// Printable list of a task's dependencies
struct Tags<'a>(&'a [String]);
impl fmt::Display for Tags<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.0.len() {
            0 => {},
            1 => write!(f, "{}", self.0[0])?,
            2.. => {
                write!(f, "{}", self.0[0])?;
                for tag in &self.0[1..] {
                    write!(f, ",{tag}")?;
                }
            },
        }
        Ok(())
    }
}

#[derive(Error, Debug)]
pub enum AppError {
    #[error(transparent)]
    ConfigError(#[from] ConfigError),
    #[error("Either a list of task ids or the -a flag must be provided")]
    MissingTaskListOrAllFlag,
    #[error(transparent)]
    GlobError(#[from] PatternError),
    #[error(transparent)]
    GraphError(#[from] GraphError),
}

type Result<T> = std::result::Result<T, AppError>;
