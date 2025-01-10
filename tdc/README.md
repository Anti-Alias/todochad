# TodoChad CLI
TodoChad CLI is a Linux cli todo list generator.
Use it to create tasks with task dependencies, forming a "graph".
Selecting a task will put it, and its sub tasks, on a todo list.

By default, database file is stored in `~/.local/share/tdc/graph.ron`
This can be configured by modifying the `~/.config/tdc/config.ron` file.

## Usage 

Let's make a task for making breakfast. 
```bash
foo@bar:~$ tdc add "Make breakfast"
0
foo@bar:~$ tdc ls
+----+----------------+----------+-------+--------------+
| id | name           | selected | order | dependencies |
+----+----------------+----------+-------+--------------+
| 0  | Make breakfast | false    |       |              |
+----+----------------+----------+-------+--------------+
```

Now, let's check our todo list...

```bash
foo@bar:~$ tdc todo 
+----+------+----------+--------------+
| id | name | selected | dependencies |
+----+------+----------+--------------+
```

Our todo list is empty because we haven't selected any tasks. Let's do that.
```bash
foo@bar:~$ tdc sel 0 
foo@bar:~$ tdc todo 
+----+----------------+----------+-------+--------------+
| id | name           | selected | order | dependencies |
+----+----------------+----------+-------+--------------+
| 0  | Make breakfast | true     |       |              |
+----+----------------+----------+-------+--------------+
```

Let's break this task into sub tasks. These are called "dependencies".
```bash
foo@bar:~$ tdc add "Get eggs"
1
foo@bar:~$ tdc add "Get milk"
2
foo@bar:~$ tdc depadd 0 1 2         # Make task 0 dependent on finishing tasks 1 and 2 first

foo@bar:~$ tdc todo
+----+----------+----------+-------+--------------+
| id | name     | selected | order | dependencies |
+----+----------+----------+-------+--------------+
| 1  | Get eggs | false    |       |              |
+----+----------+----------+-------+--------------+
| 2  | Get milk | false    |       |              |
+----+----------+----------+-------+--------------+
```

Now, we only see "Get eggs" and "Get milk" because "Make breakfast" has dependencies.
If you want to see all tasks on your todo list, use `tdc todo -a`.

Let's prioritize getting milk before getting eggs by setting an order value for each.
```bash
foo@bar:~$ tdc order 2 10
foo@bar:~$ tdc order 1 20
+----+----------+----------+-------+--------------+
| id | name     | selected | order | dependencies |
+----+----------+----------+-------+--------------+
| 2  | Get milk | false    | 10    |              |
+----+----------+----------+-------+--------------+
| 1  | Get eggs | false    | 20    |              |
+----+----------+----------+-------+--------------+
```
Note: Ordered tasks will always appear before unordered tasks on the todo list.
Now, let's start finishing tasks!

```bash
foo@bar:~$ tdc rm 2
foo@bar:~$ tdc todo 
+----+----------+----------+-------+--------------+
| id | name     | selected | order | dependencies |
+----+----------+----------+-------+--------------+
| 1  | Get eggs | false    | 20    |              |
+----+----------+----------+-------+--------------+
foo@bar:~$ tdc rm 1
foo@bar:~$ tdc todo 
+----+----------------+----------+-------+--------------+
| id | name           | selected | order | dependencies |
+----+----------------+----------+-------+--------------+
| 0  | Make breakfast | true     |       |              |
+----+----------------+----------+-------+--------------+
```

"Make breakfast" is visible once more. Let's finish this!

```bash
foo@bar:~$ tdc rm 0
foo@bar:~$ tdc todo 
+----+------+----------+--------------+
| id | name | selected | dependencies |
+----+------+----------+--------------+
```

To summarize, a todo list consists of selected tasks and their sub tasks.
When viewing your todo list, you'll only be shown tasks that have no dependencies. 
As you finish tasks with the `rm` command, more tasks will become visible.


## Command Examples

Adding a task to the database:
```bash
tdc add "Task name" 
```

Removing task 1 from the database:
```bash
tdc rm 1 
```

Listing all tasks in the database:
```bash
tdc ls 
```

Putting task 1 on the todo list:
```bash
tdc sel 1
```

Putting all tasks on the todo list:
```bash
tdc sel -a 
```

Removing task 1 from the todo list:
```bash
tdc desel 1 
```

Removing all tasks from the todo list:
```bash
tdc desel -a 
```

Making task 2 dependent on task 3:
```bash
tdc depadd 2 3
```

Removing task 2's dependency on task 3:
```bash
tdc deprm 2 3
```

Clearing all dependencies from task 3:
```bash
tdc depclear 
```

Listing tasks on the todo list, filtering out tasks that have dependencies:
```bash
tdc todo 
```

Listing all tasks on the todo list:
```bash
tdc todo -a
```

Giving task 1 an order value of 10:
```bash
tdc order 1 10 
```

Clearing the order value of task 1:
```bash
tdc order 1 
```
