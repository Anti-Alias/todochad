use slab::Slab;
use thiserror::*;
use serde::{Serialize, Deserialize};

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

    pub fn insert_with_children<D>(&mut self, mut task: Task, children: D) -> Result<TaskId>
    where 
        D: Into<Vec<TaskId>>,
    {
        let children = children.into();
        for child_id in &children {
            if !self.contains_task(*child_id) { return Err(GraphError::TaskNotFound) }
        }
        task.children = children;
        Ok(self.tasks.insert(task))
    }

    pub fn remove(&mut self, task_id: TaskId) -> Option<Task> {
        let mut task = self.tasks.try_remove(task_id)?;
        task.children.clear();
        for (_, t) in &mut self.tasks {
           t.children.retain(|tid| *tid != task_id); 
        }
        Some(task) 
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

    pub fn insert_child(&mut self, task_id: TaskId, child_id: TaskId) -> Result<()> {
        if !self.contains_task(child_id) { return Err(GraphError::TaskNotFound) }
        if self.is_reachable(child_id, task_id)? {
            return Err(GraphError::CycleDetected);
        }
        let task = self.tasks.get_mut(task_id).ok_or(GraphError::TaskNotFound)?;
        if !task.children.iter().any(|id| *id == child_id) {
            task.children.push(child_id);
        }
        Ok(()) 
    }

    pub fn remove_child(&mut self, task_id: TaskId, child_id: TaskId) -> Result<()> {
        let task = self.tasks.get_mut(task_id).ok_or(GraphError::TaskNotFound)?;
        task.children.retain(|id| *id != child_id);
        Ok(()) 
    }

    pub fn clear_children(&mut self, task_id: TaskId) -> Result<()> {
        let task = self.tasks.get_mut(task_id).ok_or(GraphError::TaskNotFound)?;
        task.children.clear();
        Ok(()) 
    }

    fn contains_task(&self, task_id: TaskId) -> bool {
        self.tasks.get(task_id).is_some()
    }

    /// Traverses the graph, starting at a given task.
    /// Returns collection of tasks 
    pub fn is_reachable(&self, task_id_a: TaskId, task_id_b: TaskId) -> Result<bool> {
        // Early checks
        let task_a = self.tasks.get(task_id_a).ok_or(GraphError::TaskNotFound)?;
        if task_id_a == task_id_b { return Ok(true) }
        if !self.contains_task(task_id_b) { return Err(GraphError::TaskNotFound) }
        // Main algorithm
        let mut visited = vec![false; self.tasks.capacity()];
        visited[task_id_a] = true;
        for child_id in task_a.children.iter().copied() {
            if self._is_reachable(child_id, task_id_b, &mut visited) {
                return Ok(true)
            }
        }
        Ok(false)
    }

    /// Traverses the graph, starting at a given task.
    /// Returns collection of tasks 
    pub fn _is_reachable(&self, task_id_a: TaskId, task_id_b: TaskId, visited: &mut [bool]) -> bool {
        if task_id_a == task_id_b { return true }
        let task_a = &self.tasks[task_id_a];
        visited[task_id_a] = true;
        for child_id in task_a.children.iter().copied() {
            if visited[child_id] { continue }
            if self._is_reachable(child_id, task_id_b, visited) {
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
        for child_id in task.children.iter().copied() {
            self.traverse(child_id, visited, result);
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Eq, PartialEq, Default, Debug)]
pub struct Task { 
    pub name: String,
    pub selected: bool,
    #[serde(rename="children")]
    children: Vec<TaskId>,
}

impl Task {

    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            selected: false,
            children: vec![],
        }
    }

    pub fn children(&self) -> &[TaskId] {
        &self.children
    }
}

pub type TaskId = usize;

#[derive(Error, Debug)]
pub enum GraphError {
    #[error("Task not found")]
    TaskNotFound,
    #[error("Task has unmet dependencies")]
    TaskDependenciesUnmet,
    #[error("Cycle detected")]
    CycleDetected,
}

pub type Result<T> = std::result::Result<T, GraphError>;


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
    fn test_insert_with_children() {
        let mut graph = Graph::new();
        let find_keys_id = graph.insert(Task::new("Find Keys"));
        let find_wallet_id  = graph.insert(Task::new("Find Wallet"));
        let get_groceries_id = graph
            .insert_with_children(Task::new("Get Groceries"), [find_keys_id, find_wallet_id])
            .unwrap();
        let get_groceries = graph.get(get_groceries_id).unwrap();
        assert_eq!(get_groceries.name.as_str(), "Get Groceries");
        assert_eq!(get_groceries.children, &[find_keys_id, find_wallet_id]);
    }

    #[test]
    fn test_insert_child() {
        let mut graph = Graph::new();
        let find_keys_id        = graph.insert(Task::new("Find Keys"));
        let find_wallet_id      = graph.insert(Task::new("Find Wallet"));
        let get_groceries_id    = graph.insert(Task::new("Get Groceries"));
        graph.insert_child(get_groceries_id, find_keys_id).unwrap();
        graph.insert_child(get_groceries_id, find_wallet_id).unwrap();
        let get_groceries = graph.get(get_groceries_id).unwrap();
        assert_eq!(get_groceries.name.as_str(), "Get Groceries");
        assert_eq!(get_groceries.children, &[find_keys_id, find_wallet_id]);
    }

    #[test]
    fn test_removal_with_children() {
        let mut graph = Graph::new();
        let find_keys_id = graph.insert(Task::new("Find Keys"));
        let find_wallet_id  = graph.insert(Task::new("Find Wallet"));
        let get_groceries_id = graph
            .insert_with_children(Task::new("Get Groceries"), [find_keys_id, find_wallet_id])
            .unwrap();
        graph.remove(find_keys_id);
        let get_groceries = graph.get(get_groceries_id).unwrap();
        assert_eq!(get_groceries.children, &[find_wallet_id]);
        graph.remove(find_wallet_id);
        let get_groceries = graph.get(get_groceries_id).unwrap();
        assert_eq!(get_groceries.children, &[]);
    }
}
