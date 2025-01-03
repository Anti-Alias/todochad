use slab::Slab;
use thiserror::*;
use derive_more::Display;
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
            if task.status() != TaskStatus::Selected { continue }
            self.traverse(task_id, &mut visited, &mut result);
        }
        result
    }

    pub fn set_status(&mut self, task_id: TaskId, desired_status: TaskStatus) -> Result<()> {
        let task = self.tasks.get_mut(task_id).ok_or(GraphError::TaskNotFound)?;
        match (desired_status, task.status) {
            (TaskStatus::Selected, TaskStatus::Finished)  => return Err(GraphError::CannotSelectFinishedTask),
            (TaskStatus::Deselected, TaskStatus::Finished) => return Err(GraphError::CannotDeselectFinishedTask),
            (TaskStatus::Finished, TaskStatus::Finished) => return Err(GraphError::CannotFinishFinishedTask),
            (TaskStatus::Finished, _) => {
                if !self.has_met_dependencies(task_id).unwrap() {
                    return Err(GraphError::TaskDependenciesUnmet);
                }
            } 
            _ => {}
        }
        let task = self.tasks.get_mut(task_id).unwrap();
        task.status = desired_status;
        Ok(())
    }

    pub fn select_all(&mut self) {
        for (_task_id, task) in &mut self.tasks {
            if task.status == TaskStatus::Finished { continue }
            task.status = TaskStatus::Selected;
        }
    }

    pub fn deselect_all(&mut self) {
        for (_task_id, task) in &mut self.tasks {
            if task.status == TaskStatus::Finished { continue }
            task.status = TaskStatus::Deselected;
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

    pub fn has_met_dependencies(&self, task_id: TaskId) -> Result<bool> {
        let task = self.tasks.get(task_id).ok_or(GraphError::TaskNotFound)?;
        if task.status == TaskStatus::Finished { return Ok(false) }
        for child_id in task.children.iter().copied() {
            let child = &self.tasks[child_id];
            if child.status != TaskStatus::Finished {
                return Ok(false);
            }
        }
        Ok(true)
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
    status: TaskStatus,
    #[serde(rename="children")]
    children: Vec<TaskId>,
}

impl Task {

    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            status: TaskStatus::default(),
            children: vec![],
        }
    }

    pub fn status(&self) -> TaskStatus {
        self.status
    }

    pub fn children(&self) -> &[TaskId] {
        &self.children
    }
}

#[derive(Serialize, Deserialize, Display, Copy, Clone, Eq, PartialEq, Default, Debug)]
pub enum TaskStatus {
    #[default]
    Deselected,
    Selected,
    Finished,
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
    #[error("Cannot select finished task")]
    CannotSelectFinishedTask,
    #[error("Cannot deselect finished task")]
    CannotDeselectFinishedTask,
    #[error("Cannot finish an already finished task")]
    CannotFinishFinishedTask,
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
