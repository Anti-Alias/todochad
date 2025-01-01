use slab::Slab;
use thiserror::*;
use serde::{Serialize, Deserialize};

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct Graph {
   todos: Slab<Todo>,
}

impl Graph {

    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, todo: Todo) -> TodoId {
        self.todos.insert(todo)
    }
    
    pub fn insert_with_dependencies<D>(&mut self, mut todo: Todo, dependencies: D) -> Result<TodoId>
    where 
        D: Into<Vec<TodoId>>,
    {
        let dependencies = dependencies.into();
        for dep_id in &dependencies {
            if !self.contains_todo(*dep_id) { return Err(GraphError::TodoNotFound) }
        }
        todo.dependencies = dependencies;
        Ok(self.todos.insert(todo))
    }

    pub fn remove(&mut self, todo_id: TodoId) -> Option<Todo> {
        let mut todo = self.todos.try_remove(todo_id)?;
        todo.dependencies.clear();
        for (_, t) in &mut self.todos {
           t.dependencies.retain(|tid| *tid != todo_id); 
        }
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

    pub fn len(&self) -> usize {
        self.todos.len()
    }

    pub fn is_empty(&self) -> bool {
        self.todos.is_empty()
    }

    pub fn insert_dependency(&mut self, todo_id: TodoId, dep_id: TodoId) -> Result<()> {
        if !self.contains_todo(dep_id) { return Err(GraphError::TodoNotFound) }
        let todo = self.todos.get_mut(todo_id).ok_or(GraphError::TodoNotFound)?;
        if !todo.dependencies.iter().any(|id| *id == dep_id) {
            todo.dependencies.push(dep_id);
        }
        Ok(()) 
    }

    pub fn remove_dependency(&mut self, todo_id: TodoId, dep_id: TodoId) -> Result<()> {
        let todo = self.todos.get_mut(todo_id).ok_or(GraphError::TodoNotFound)?;
        todo.dependencies.retain(|id| *id != dep_id);
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

#[derive(Serialize, Deserialize, Clone, Eq, PartialEq, Default, Debug)]
pub struct Todo { 
    pub name: String,
    dependencies: Vec<TodoId>,
}

impl Todo {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            dependencies: vec![],
        }
    }
}

pub type TodoId = usize;


#[cfg(test)]
mod test {
    use crate::{ Graph, Todo };

    #[test]
    fn test_insertion_and_retrieval() {
        let mut graph = Graph::new();
        let find_keys_id = graph.insert(Todo::new("Find Keys"));
        let find_wallet_id  = graph.insert(Todo::new("Find Wallet"));
        let find_keys = graph.get(find_keys_id).unwrap();
        let find_wallet = graph.get(find_wallet_id).unwrap();
        assert_eq!(find_keys, &Todo::new("Find Keys"));
        assert_eq!(find_wallet, &Todo::new("Find Wallet"));
    }

    #[test]
    fn test_removal() {
        let mut graph = Graph::new();
        let find_keys_id = graph.insert(Todo::new("Find Keys"));
        let find_wallet_id  = graph.insert(Todo::new("Find Wallet"));
        let find_keys = graph.remove(find_keys_id);
        let find_wallet = graph.get(find_wallet_id);
        let find_keys_ref = graph.get(find_keys_id);
        assert_eq!(find_keys, Some(Todo::new("Find Keys")));
        assert_eq!(find_wallet, Some(&Todo::new("Find Wallet")));
        assert_eq!(find_keys_ref, None); 
    }

    #[test]
    fn test_insert_with_dependencies() {
        let mut graph = Graph::new();
        let find_keys_id = graph.insert(Todo::new("Find Keys"));
        let find_wallet_id  = graph.insert(Todo::new("Find Wallet"));
        let get_groceries_id = graph
            .insert_with_dependencies(Todo::new("Get Groceries"), [find_keys_id, find_wallet_id])
            .unwrap();
        let get_groceries = graph.get(get_groceries_id).unwrap();
        assert_eq!(get_groceries.name.as_str(), "Get Groceries");
        assert_eq!(get_groceries.dependencies, &[find_keys_id, find_wallet_id]);
    }

    #[test]
    fn test_insert_dependency() {
        let mut graph = Graph::new();
        let find_keys_id        = graph.insert(Todo::new("Find Keys"));
        let find_wallet_id      = graph.insert(Todo::new("Find Wallet"));
        let get_groceries_id    = graph.insert(Todo::new("Get Groceries"));
        graph.insert_dependency(get_groceries_id, find_keys_id).unwrap();
        graph.insert_dependency(get_groceries_id, find_wallet_id).unwrap();
        let get_groceries = graph.get(get_groceries_id).unwrap();
        assert_eq!(get_groceries.name.as_str(), "Get Groceries");
        assert_eq!(get_groceries.dependencies, &[find_keys_id, find_wallet_id]);
    }

    #[test]
    fn test_removal_with_dependencies() {
        let mut graph = Graph::new();
        let find_keys_id = graph.insert(Todo::new("Find Keys"));
        let find_wallet_id  = graph.insert(Todo::new("Find Wallet"));
        let get_groceries_id = graph
            .insert_with_dependencies(Todo::new("Get Groceries"), [find_keys_id, find_wallet_id])
            .unwrap();
        graph.remove(find_keys_id);
        let get_groceries = graph.get(get_groceries_id).unwrap();
        assert_eq!(get_groceries.dependencies, &[find_wallet_id]);
        graph.remove(find_wallet_id);
        let get_groceries = graph.get(get_groceries_id).unwrap();
        assert_eq!(get_groceries.dependencies, &[]);
    }
}
