# TodoChad
TodoChad is a todo-list generator.
Use it to create tasks, task dependencies, and select tasks you want to prioritize. 
TodoChad will take the task "graph" you create and use it to generate a flat todo list
with only the tasks that are doable at the moment.
As you finish tasks on your todo list, tasks previously hidden due to having unmet dependencies become visible.

Database file is stored in `~/.local/share/tdc/graph.ron`

## Examples

Adding tasks, then listing all tasks. 
```bash
foo@bar:~$ tdc add "Make breakfast"
0
foo@bar:~$ tdc add "Get eggs"
1
foo@bar:~$ tdc add "Get milk"
2
foo@bar:~$ tdc ls
+----+----------------+----------+----------+--------+--------------+
| id | name           | selected | finished | doable | dependencies |
+----+----------------+----------+----------+--------+--------------+
| 0  | Make breakfast | false    | false    | true   |              |
+----+----------------+----------+----------+--------+--------------+
| 1  | Get eggs       | false    | false    | true   |              |
+----+----------------+----------+----------+--------+--------------+
| 2  | Get milk       | false    | false    | true   |              |
+----+----------------+----------+----------+--------+--------------+
```

Make finishing "Make breakfast" be dependent on finishing "Get eggs" and "Get milk" first.
This makes "Make breakfast" no longer "doable".
```bash
foo@bar:~$ tdc depadd 0 1 
foo@bar:~$ tdc depadd 0 2 
foo@bar:~$ tdc ls
+----+----------------+----------+----------+--------+--------------+
| id | name           | selected | finished | doable | dependencies |
+----+----------------+----------+----------+--------+--------------+
| 0  | Make breakfast | false    | false    | false  | 1,2          |
+----+----------------+----------+----------+--------+--------------+
| 1  | Get eggs       | false    | false    | true   |              |
+----+----------------+----------+----------+--------+--------------+
| 2  | Get milk       | false    | false    | true   |              |
+----+----------------+----------+----------+--------+--------------+
```

Selecting "Make breakfast", putting it and its dependencies on the todo list.
As far as TodoChad is concerned, "Make breakfast" is our one goal in life, and its role is to get us there!
```bash
foo@bar:~$ tdc sel 0
```

Show entire todo list, even tasks with unmet dependencies.
Tasks that are doable will be shown first.
```bash
foo@bar:~$ tdc todo -a
+----+----------------+----------+----------+--------+--------------+
| id | name           | selected | finished | doable | dependencies |
+----+----------------+----------+----------+--------+--------------+
| 1  | Get eggs       | false    | false    | true   |              |
+----+----------------+----------+----------+--------+--------------+
| 2  | Get milk       | false    | false    | true   |              |
+----+----------------+----------+----------+--------+--------------+
| 0  | Make breakfast | true     | false    | false  | 1,2          |
+----+----------------+----------+----------+--------+--------------+
```

Show todo list, filtering out tasks that aren't doable due to having unmet dependencies.
In this case, "Make breakfast" will not be shown until "Get eggs" and "Get milk" are finished.
```bash
foo@bar:~$ tdc todo 
+----+----------+----------+----------+--------+--------------+
| id | name     | selected | finished | doable | dependencies |
+----+----------+----------+----------+--------+--------------+
| 1  | Get eggs | false    | false    | true   |              |
+----+----------+----------+----------+--------+--------------+
| 2  | Get milk | false    | false    | true   |              |
+----+----------+----------+----------+--------+--------------+
```


Finishing tasks one by one.
```bash
foo@bar:~$ tdc finish 1 
foo@bar:~$ tdc todo 
+----+----------+----------+----------+--------+--------------+
| id | name     | selected | finished | doable | dependencies |
+----+----------+----------+----------+--------+--------------+
| 2  | Get milk | false    | false    | true   |              |
+----+----------+----------+----------+--------+--------------+
foo@bar:~$ tdc finish 2 
foo@bar:~$ tdc todo 
+----+----------------+----------+----------+--------+--------------+
| id | name           | selected | finished | doable | dependencies |
+----+----------------+----------+----------+--------+--------------+
| 0  | Make breakfast | true     | false    | true   | 1,2          |
+----+----------------+----------+----------+--------+--------------+
foo@bar:~$ tdc finish 0 
foo@bar:~$ tdc todo 
+----+---------------+----------+----------+--------+--------------+
| id | name          | selected | finished | doable | dependencies |
+----+---------------+----------+----------+--------+--------------+
```
