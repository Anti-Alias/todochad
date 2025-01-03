# TodoChad
TodoChad is a todo-list generator.
You create tasks, create task dependencies, and mark tasks you want to prioritize. 
TodoChad will only show you the doable tasks at any given time, making task management more manageable. 

Adding tasks.
```bash
foo@bar:~$ tdc add "Get groceries"
0
foo@bar:~$ tdc add "Get eggs"
1
foo@bar:~$ tdc add "Get milk"
2
```

Setting task dependencies. 
Make "Get groceries" be dependent on "Get eggs" and "Get milk".
Tasks can only be finished when all dependent tasks are finished.
```bash
foo@bar:~$ tdc depadd 0 1 
foo@bar:~$ tdc depadd 0 2 
```

Listing all tasks in database.
```bash
foo@bar:~$ tdc ls 
+----+---------------+----------+----------+--------+--------------+
| id | name          | selected | finished | doable | dependencies |
+----+---------------+----------+----------+--------+--------------+
| 0  | Get groceries | false    | false    | false  | 1,2          |
+----+---------------+----------+----------+--------+--------------+
| 1  | Get eggs      | false    | false    | true   |              |
+----+---------------+----------+----------+--------+--------------+
| 2  | Get milk      | false    | false    | true   |              |
+----+---------------+----------+----------+--------+--------------+
```

Selecting ""Get groceries"" task, putting it, and its dependencies on the todo list.
```bash
foo@bar:~$ tdc sel 0
```

Shows todo list, filtering out tasks that aren't doable.
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

Shows entire todo list 
```bash
foo@bar:~$ tdc todo -a
+----+---------------+----------+----------+--------+--------------+
| id | name          | selected | finished | doable | dependencies |
+----+---------------+----------+----------+--------+--------------+
| 1  | Get eggs      | false    | false    | true   |              |
+----+---------------+----------+----------+--------+--------------+
| 2  | Get milk      | false    | false    | true   |              |
+----+---------------+----------+----------+--------+--------------+
| 0  | Get groceries | true     | false    | false  | 1,2          |
+----+---------------+----------+----------+--------+--------------+

