## Current Options

### [DBToaster](https://dbtoaster.github.io/docs_sql.html)
### Parameterized Queries

```
SQL -> Calculus/Plan/M3/K3? -> Backend -> C++/Scala 
```
Has some recent maintenance, but no active work in a few years.
- Only support a stream of inserts as a relation (no update or deletion)
- Inserts are not timestamped
- Contains C++, Scala backends. Written in OCaml
- Can embed generated code in applications
- Streaming allowed from CSV, from application and special trading type (used for a demo)

*Key Disadvantage*: Insert only, fundamentally a view rather than DB, cannot do parameterised queries 

### [Materialize](https://materialize.com/)
Standalone system that does IVM sourced from database logs.
- Supports all actions on the database
- Based on previous Naiad work at Microsoft Research
- Open source [TimelyDataflow](https://github.com/TimelyDataflow) framework that includes a differential dataflow framework.

*key disadvantage*: Not embedded
Based on SQL-92, so cannot do
### [HyPer](https://www.hyper-db.de/)
In-Memory database with codegen.
- uses inbuilt llvm compiler to generate machine code for queries, advertised speed better than VoltDB
- project continued as [umbra](https://umbra-db.com/) 

*key disadvantage*: Not embedded, cannot take advantage of known queries beyond caching codegen

### SingleStore/MemSQL
Very cool newSQL database.
- Proprietary database, backedn by disk.
- Use of codegen, mvcc (lock free), and in-memory tech from MemSQL for performance

*key disadvantage*: Not embedded, cannot take advantage of known queries

### [Hekaton](https://www.microsoft.com/en-us/research/wp-content/uploads/2013/06/Hekaton-Sigmod2013-final.pdf)
An OLTP in-memory database for Microsoft SQL Server.


## Misc
Codegen in databases with [coat](https://github.com/tetzank/coat) and discussed in the author's article [here](https://tetzank.github.io/posts/codegen-in-databases/).