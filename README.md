# TodoChad
TodoChad is a cli todo list generator.
Use it to create tasks with task dependencies, forming a "graph".
Selecting a task will put it and its dependencies on a todo list.

Database file is stored in `~/.local/share/tdc/graph.ron`

## Usage 

Let's make a task for making breakfast. 
```bash
foo@bar:~$ tdc add "Make breakfast"     # Adds a task
0
foo@bar:~$ tdc ls                       # Lists all tasks
+----+----------------+----------+--------------+
| id | name           | selected | dependencies |
+----+----------------+----------+--------------+
| 0  | Make breakfast | false    |              |
+----+----------------+----------+--------------+
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
+----+----------------+----------+--------------+
| id | name           | selected | dependencies |
+----+----------------+----------+--------------+
| 0  | Make breakfast | false    |              |
+----+----------------+----------+--------------+
```

Let's add some sub tasks that must be completed before "Make breakfast" can be completed.
These are called "dependencies".
```bash
foo@bar:~$ tdc add "Get eggs"
1
foo@bar:~$ tdc add "Get milk"
2
foo@bar:~$ tdc depadd 0 1 2         # Make task 0 dependent on finishing tasks 1 and 2 first

foo@bar:~$ tdc todo
+----+----------+----------+--------------+
| id | name     | selected | dependencies |
+----+----------+----------+--------------+
| 1  | Get eggs | false    |              |
+----+----------+----------+--------------+
| 2  | Get milk | false    |              |
+----+----------+----------+--------------+
```

We only see "Get eggs" and " now because "Make breakfast" has dependencies.
Let's cross stuff off our todo list now!

```bash
foo@bar:~$ tdc rm 1
foo@bar:~$ tdc todo 
+----+----------+----------+--------------+
| id | name     | selected | dependencies |
+----+----------+----------+--------------+
| 2  | Get milk | false    |              |
+----+----------+----------+--------------+
foo@bar:~$ tdc rm 2 
foo@bar:~$ tdc todo 
+----+----------------+----------+--------------+
| id | name           | selected | dependencies |
+----+----------------+----------+--------------+
| 0  | Make breakfast | false    |              |
+----+----------------+----------+--------------+
```

We removed the "Get eggs" and "Get milk" tasks, making "Make breakfast" visible again!
Let's finish this!

```bash
foo@bar:~$ tdc rm 0
foo@bar:~$ tdc todo 
+----+------+----------+--------------+
| id | name | selected | dependencies |
+----+------+----------+--------------+
```

To summarize, a todo list consists of all selected tasks and all of their sub tasks.
when viewing your todo list, you'll only be shown tasks that are "doable".
As you finish tasks with the rm command, more tasks will become visible again.
This makes TodoChad great for defining stretch goals.
