# JustAct: Defining Actions for Agents
This crate implements the Acting language (`.act`), which is a declarative programming language that programs agents in JustAct in a simple way.


## Language
The language is structured as a list of declarative rules, with _triggers_ on the left and _actions_ on the right. Syntactically, this has the form:
```acting
[<LABEL>:] on <TRIGGER> do <ACTION>[ do <ACTION>[ ...]] [;].
```

where `<LABEL>` is an optional string value that names the rule; `<TRIGGER>` is one of the triggers described [below](#triggers); and `<ACTION>` is one of the actions [below](#actions).

The language supports C-style single- and multi-line comments (`// ...` and `/* ... */`).

### Triggers
The following triggers are valid triggers for agents to act on:
- `start`: Initial triggers that will occur at startup time. Their order is arbitrary.
- `time <EXPR>`: Some expression over the current timestep (`now`) that, once a time is reached that will make the expression true, will trigger the action.
  - Allowed expressions are either `now` to refer the current timestep; a constant integer (e.g., `42`) OR an operator.
  - Allowed operators are `+`, `-`, `*`, `/`, `%`, `&&`, `||`, `==`, `!=`, `>`, `>=`, `<` or `<=`. Parenthesis may also be used.
- `message <ID>`: Some message identifier, e.g., `15` or a string value `"foo"`, that, once received by the agent, will trigger the action.
  - Note that message sent by the agent itself will also trigger the value.
- `message by <AUTHOR>`: Some author name (as a string value, `"amy"`), which will trigger every time this agent receives a message from that agent.
- `message contains <REGEX>`: A regular expression (using [this](TODO) syntax) that will match any incoming message. The action will be triggered if the regular expression matches.

### Actions
The following actions may be executed when triggers are triggered:
- `nop`: Do nothing.
- `trigger <INDEX|LABEL>`: Triggers the rule indicated by the given index OR label. Labels must be string values (e.g., `"foo"`).
- `tick`: Move to the next timestep. Only possible if the underlying simulation scheme allows this agent to do so.
    - This propagates the validity of the agreement valid in the previous timestep.
- `agree <ID> [<LANG>] { ... }`: Define a new agreement that is valid at a next time step. The contents of the curly brackets can be anything except curly brackets. If desired, an optional language specifier can be given to compile the language before submission to assert validity.
- `state [to <AGENT>] <ID> [\<<LANG>\>] { ... }`: Sends a message to everybody or, if specified, a specific agent. The contents of the curly brackets can be anything except curly brackets. If desired, an optional language specifier can be given to compile the language before submission to assert validity.
- `enact [to <AGENT>] <ID>[, <ID> [...]]`: Enacts a set of messages (referred to by IDs, separated by commas) as an action. If specified, sends it only to a specific agent instead of everybody. Note that the framework will consider any action invalid unless it includes the current agreement and enacted statement.

### Example
Implementation for the `amy` agent of the example in the paper \[2\]:
```acting
// wait until 's2' is stated
on message "s2"
    // State the action Amy wants to enact
    do state "s3" <datalog> {
        ctl_accesses(amy, x_rays).
    }
    // Enact it
    do enact "s1", "s2", "s3".
```

Another example, the consortium agent for that example:
```acting
on start
    // Publish the agreement on start
    agree "s1" <datalog> {
        owns(administrator, Data) :- ctl_accesses(Accessor, Data).
        error :- ctl_accesses(Accessor, Data), owns(Owner, Data), not ctl_authorises(Owner, Accessor, Data).
    }
```
