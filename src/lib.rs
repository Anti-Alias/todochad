use slab::Slab;
use std::collections::HashMap;
use thiserror::*;
use serde::{Serialize, Deserialize};

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct Graph {
   todos: Slab<Todo>,
   dependencies: HashMap<TodoId, Vec<TodoId>>,
}

impl Graph {

    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, todo: Todo) -> TodoId {
        self.todos.insert(todo)
    }

    pub fn remove(&mut self, todo_id: TodoId) -> Option<Todo> {
        let todo = self.todos.try_remove(todo_id)?;
        self.dependencies.retain(|parent_id, child_id| todo_id != *parent_id && todo_id != *child_id);
        Some(todo) 
    }

    pub fn get(&self, todo_id: TodoId) -> Option<&Todo> {
        self.todos.get(todo_id)
    }

    pub fn get_mut(&mut self, todo_id: TodoId) -> Option<&mut Todo> {
        self.todos.get_mut(todo_id)
    }

    pub fn iter(&self) -> impl Iterator<Item = (TodoId, &Todo)> {
        self.todos.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (TodoId, &mut Todo)> {
        self.todos.iter_mut()
    }

    pub fn traverse(&self) -> Vec<(TodoId, Todo)> {
        let mut result = vec![];
        result
    }


    pub fn insert_dependency(&mut self, parent_id: TodoId, child_id: TodoId) -> Result<()> {
        if !self.contains_todo(parent_id) || !self.contains_todo(child_id) {
            return Err(GraphError::TodoNotFound);
        }
        self.dependencies.insert(parent_id, child_id);
        Ok(()) 
    }

    pub fn remove_dependency(&mut self, parent_id: TodoId, child_id: TodoId) -> Result<()> {
        if !self.contains_todo(parent_id) || !self.contains_todo(child_id) {
            return Err(GraphError::TodoNotFound);
        }
        self.dependencies.insert(parent_id, child_id);
        Ok(()) 
    }

    fn contains_todo(&self, todo_id: TodoId) -> bool {
        self.todos.iter().any(|(tid, _)| tid == todo_id)
    }
}

#[derive(Error, Debug)]
pub enum GraphError {
    #[error("Todo not found")]
    TodoNotFound,
}

pub type Result<T> = std::result::Result<T, GraphError>;

#[derive(Serialize, Deserialize, Clone, Default, Debug)]
pub struct Todo { pub name: String }
pub type TodoId = usize;
