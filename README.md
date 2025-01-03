# TodoChad
TodoChad is a todo-list generator.
Use it to create tasks, task dependencies, and select tasks you want to prioritize. 
TodoChad will take the task "graph" you create and use it to generate a flat todo list
with only the tasks that are doable at the moment.
At you finish tasks on your todo list, tasks previously hidden due to having unmet dependencies become visible.

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

Selecting the "Get groceries" task, putting it, and its dependencies on the todo list.
Multiple tasks can be selected at once, if you want to prioritize more than one thing.
```bash
foo@bar:~$ tdc sel 0 ```

Shows todo list, filtering out tasks that aren't "doable" due to unmet dependencies.
"Get groceries" will not be shown until "Get eggs" and "Get milk" are finished.
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

Show entire todo list, even tasks with unmet dependencies.
Tasks that are doable will be shown first.
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
```

Finishing tasks, one by one.
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
+----+---------------+----------+----------+--------+--------------+
| id | name          | selected | finished | doable | dependencies |
+----+---------------+----------+----------+--------+--------------+
| 0  | Get groceries | true     | false    | true   | 1,2          |
+----+---------------+----------+----------+--------+--------------+
foo@bar:~$ tdc finish 0 
foo@bar:~$ tdc todo 
+----+---------------+----------+----------+--------+--------------+
| id | name          | selected | finished | doable | dependencies |
+----+---------------+----------+----------+--------+--------------+
```
