use std::{env, fs};
use std::path::{PathBuf, Path};
use thiserror::Error;
use todochad::Graph;

const APP_NAME: &str        = "todochad";
const GRAPH_FILE_NAME: &str = "graph.ron";

fn main() -> Result<()> {
    let graph_path = graph_path()?;
    let mut graph = load_graph(&graph_path)?;
    save_graph(&graph_path, &graph)?;
    Ok(())
}

fn load_graph(graph_path: &Path) -> Result<Graph> {
    match fs::exists(graph_path) { 
        Err(_) => Err(AppError::GraphReadError),
        Ok(false) => Ok(Graph::new()),
        Ok(true) => {
            let graph_str = fs::read_to_string(graph_path).or(Err(AppError::GraphReadError))?;
            let graph = ron::from_str(&graph_str).or(Err(AppError::GraphParseError))?;
            Ok(graph)
        },
    }
}

fn save_graph(graph_path: &Path, graph: &Graph) -> Result<()> {
    let graph_str = ron::to_string(graph).expect("Failed to serialize graph");
    fs::write(graph_path, graph_str).or(Err(AppError::GraphWriteError))?;
    Ok(())
}

/// Determines path of graph file.
/// Creates directory structure along the way if it does not exist.
fn graph_path() -> Result<PathBuf> {
    let home = env::var("HOME").or(Err(AppError::HomeDirError))?;
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
}

type Result<T> = std::result::Result<T, AppError>;
