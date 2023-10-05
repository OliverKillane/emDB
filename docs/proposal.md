# Project proposal for Imperial College

## Embedded Codegen Database using known queries for optimisation

Many types of applications need to store & interact with large amounts of complex data (e.g with references & upholding invariants) but do not require long term persistence, and have all queries known prior to compiling the application.

Examples:

<ul>
  <li>Short term/daily databases for services (e.g trade reporting systems)</li>
  <li>Complex ephemeral state for servers (e.g online multiplayer games, alerting systems)</li>
  <li>Fast analysis of sections of historical data (e.g financial simulations)</li>
</ul>

For these applications several suboptimal solutions exist.

<ul>
<li>Using a normal database, incurring significant cost (in maintenance time, ease of changes in development & efficiency) for unused features (long term persistence, flexibility)</li>
<li>Writing the data management by hand, requiring more developer time to ensure correctness & to ptimise, and making the cost of restructuring, re-verifying and re-optimising changes.</li>
<li>Using dataframes (e.g pandas, polars) requires less time than a by-hand implementation, but developers still need to test & optimise themselves.</li>
<li>Using an embedded database (e.g SQLite, duckDB, derby) to include a database within the application itself. Much correctness is verified by the database engine</li>
</ul>
None of these options take full advantage of queries being known when the application is compiled.

The goal of this project is to build a tool that can use the set of known queries to generate code for an embedded database. The tools should be evaluated to compare the affect of optimisations, and more broadly to compare against the aforementioned alternative solutions.

The tool should:

<ul>
<li>Move semantic analysis & query optimisation from runtime to application compile time.</li>
<li>Use queries to make more optimal data structure & index choices for tables.</li>
<li>Generate code for the database at application compile time.</li>
<li>Provide an easy to use api for users that allows them to integrate with their own code.</li>
</ul>
