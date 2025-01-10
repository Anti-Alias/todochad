use slab::Slab;
use thiserror::*;
use serde::{Serialize, Deserialize};
use ron::ser::PrettyConfig;
use std::{fs, fmt};
use crate::Config;

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct Graph {
   tasks: Slab<Task>,
}

impl Graph {

    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, task: Task) -> TaskId {
        self.tasks.insert(task)
    }

    pub fn remove(&mut self, task_id: TaskId) -> Option<Task> {
        let mut task = self.tasks.try_remove(task_id)?;
        task.dependencies.clear();
        for (_, t) in &mut self.tasks {
           t.dependencies.retain(|tid| *tid != task_id);
        }
        Some(task)
    }

    pub fn clear(&mut self) {
        self.tasks.clear();
    }

    pub fn get(&self, task_id: TaskId) -> Option<&Task> {
        self.tasks.get(task_id)
    }

    pub fn get_mut(&mut self, task_id: TaskId) -> Option<&mut Task> {
        self.tasks.get_mut(task_id)
    }

    pub fn traverse_selected(&self) -> Vec<(TaskId, &Task)> {
        let mut result = vec![];
        let mut visited = vec![false; self.tasks.capacity()];
        for (task_id, task) in &self.tasks {
            if !task.selected { continue }
            self.traverse(task_id, &mut visited, &mut result);
        }
        result
    }

    pub fn set_selected(&mut self, task_id: TaskId, selected: bool) -> Result<()> {
        let task = self.tasks.get_mut(task_id).ok_or(GraphError::TaskNotFound)?;
        task.selected = selected;
        Ok(())
    }

    pub fn set_selected_all(&mut self, selected: bool) {
        for (_task_id, task) in &mut self.tasks {
            task.selected = selected;
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = (TaskId, &Task)> {
        self.tasks.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (TaskId, &mut Task)> {
        self.tasks.iter_mut()
    }

    pub fn len(&self) -> usize {
        self.tasks.len()
    }

    pub fn is_empty(&self) -> bool {
        self.tasks.is_empty()
    }

    pub fn insert_dependency(&mut self, task_id: TaskId, dependency_id: TaskId) -> Result<()> {
        if !self.contains_task(dependency_id) { return Err(GraphError::TaskNotFound) }
        if self.is_reachable(dependency_id, task_id)? {
            return Err(GraphError::CycleDetected);
        }
        let task = self.tasks.get_mut(task_id).ok_or(GraphError::TaskNotFound)?;
        if !task.dependencies.iter().any(|id| *id == dependency_id) {
            task.dependencies.push(dependency_id);
        }
        Ok(())
    }

    pub fn remove_dependency(&mut self, task_id: TaskId, dependency_id: TaskId) -> Result<()> {
        let task = self.tasks.get_mut(task_id).ok_or(GraphError::TaskNotFound)?;
        task.dependencies.retain(|id| *id != dependency_id);
        Ok(())
    }

    pub fn clear_dependencies(&mut self, task_id: TaskId) -> Result<()> {
        let task = self.tasks.get_mut(task_id).ok_or(GraphError::TaskNotFound)?;
        task.dependencies.clear();
        Ok(())
    }

    fn contains_task(&self, task_id: TaskId) -> bool {
        self.tasks.get(task_id).is_some()
    }

    /// Traverses the graph, starting at a given task.
    /// Returns collection of tasks.
    pub fn is_reachable(&self, task_id_a: TaskId, task_id_b: TaskId) -> Result<bool> {
        // Early checks
        let task_a = self.tasks.get(task_id_a).ok_or(GraphError::TaskNotFound)?;
        if task_id_a == task_id_b { return Ok(true) }
        if !self.contains_task(task_id_b) { return Err(GraphError::TaskNotFound) }
        // Main algorithm
        let mut visited = vec![false; self.tasks.capacity()];
        visited[task_id_a] = true;
        for dependency_id in task_a.dependencies.iter().copied() {
            if self._is_reachable(dependency_id, task_id_b, &mut visited) {
                return Ok(true)
            }
        }
        Ok(false)
    }

    /// Traverses the graph, starting at a given task.
    /// Returns collection of tasks.
    pub fn _is_reachable(&self, task_id_a: TaskId, task_id_b: TaskId, visited: &mut [bool]) -> bool {
        if task_id_a == task_id_b { return true }
        let task_a = &self.tasks[task_id_a];
        visited[task_id_a] = true;
        for dependency_id in task_a.dependencies.iter().copied() {
            if visited[dependency_id] { continue }
            if self._is_reachable(dependency_id, task_id_b, visited) {
                return true
            }
        }
        false
    }

    fn traverse<'a>(
        &'a self,
        task_id: TaskId,
        visited: &mut [bool],
        result: &mut Vec<(TaskId, &'a Task)>,
    ) {
        if visited[task_id] { return }
        visited[task_id] = true;
        let task = &self.tasks[task_id];
        result.push((task_id, task));
        for dependency_id in task.dependencies.iter().copied() {
            self.traverse(dependency_id, visited, result);
        }
    }


    pub fn load(config: &Config) -> Result<Graph> {
        if fs::exists(&config.graph_path)? {
            let graph_string = std::fs::read_to_string(&config.graph_path)?;
            let graph = ron::de::from_str(&graph_string).map_err(|_| GraphError::GraphParseError)?;
            Ok(graph)
        }
        else {
            Ok(Graph::default())
        }
    }

    pub fn save(&self, config: &Config) -> Result<()> {
        if let Some(parent_dir) = config.graph_path.parent() {
            fs::create_dir_all(parent_dir)?;
        }
        let string = ron::ser::to_string_pretty(self, PrettyConfig::default()).expect("Failed to serialize graph");
        fs::write(&config.graph_path, string)?;
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Clone, Eq, PartialEq, Default, Debug)]
pub struct Task {
    pub name: String,
    pub selected: bool,
    pub order: TaskOrder,
    dependencies: Vec<TaskId>,
}

impl Task {

    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            selected: false,
            order: TaskOrder::default(),
            dependencies: vec![],
        }
    }

    pub fn dependencies(&self) -> &[TaskId] {
        &self.dependencies
    }
}

#[derive(Serialize, Deserialize, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Default, Debug)]
pub enum TaskOrder {
    Order(i32),
    #[default]
    Last,
}

impl fmt::Display for TaskOrder {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let TaskOrder::Order(order) = self {
            write!(f, "{order}")?;
        }
        Ok(())
    }
}

pub type TaskId = usize;

#[derive(Error, Debug)]
pub enum GraphError {
    #[error("Failed to parse graph file")]
    GraphParseError,
    #[error(transparent)]
    IOError(#[from] std::io::Error),
    #[error("Task not found")]
    TaskNotFound,
    #[error("Task has unmet dependencies")]
    TaskDependenciesUnmet,
    #[error("Cycle detected")]
    CycleDetected,
}

type Result<T> = std::result::Result<T, GraphError>;


#[cfg(test)]
mod test {
    use crate::{ Graph, Task };

    #[test]
    fn test_insertion_and_retrieval() {
        let mut graph = Graph::new();
        let find_keys_id = graph.insert(Task::new("Find Keys"));
        let find_wallet_id  = graph.insert(Task::new("Find Wallet"));
        let find_keys = graph.get(find_keys_id).unwrap();
        let find_wallet = graph.get(find_wallet_id).unwrap();
        assert_eq!(find_keys, &Task::new("Find Keys"));
        assert_eq!(find_wallet, &Task::new("Find Wallet"));
    }

    #[test]
    fn test_insert_dependency() {
        let mut graph = Graph::new();
        let find_keys_id = graph.insert(Task::new("Find Keys"));
        let find_wallet_id = graph.insert(Task::new("Find Wallet"));
        let get_groceries_id = graph.insert(Task::new("Get Groceries"));
        graph.insert_dependency(get_groceries_id, find_keys_id).unwrap();
        graph.insert_dependency(get_groceries_id, find_wallet_id).unwrap();
        let get_groceries = graph.get(get_groceries_id).unwrap();
        assert_eq!(get_groceries.name.as_str(), "Get Groceries");
        assert_eq!(get_groceries.dependencies, &[find_keys_id, find_wallet_id]);
    }

    #[test]
    fn test_removal_with_dependencies() {
        let mut graph = Graph::new();
        let find_keys_id = graph.insert(Task::new("Find Keys"));
        let find_wallet_id = graph.insert(Task::new("Find Wallet"));
        let get_groceries_id = graph.insert(Task::new("Get Groceries"));
        graph.insert_dependency(get_groceries_id, find_keys_id).unwrap();
        graph.insert_dependency(get_groceries_id, find_wallet_id).unwrap();
        graph.remove(find_keys_id);
        let get_groceries = graph.get(get_groceries_id).unwrap();
        assert_eq!(get_groceries.dependencies, &[find_wallet_id]);
        graph.remove(find_wallet_id);
        let get_groceries = graph.get(get_groceries_id).unwrap();
        assert_eq!(get_groceries.dependencies, &[]);
    }
}
