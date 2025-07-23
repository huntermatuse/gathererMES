# what it is

this is kinda a weird project and will be shifting around a lot as i figure out the layout. this file should serve as a reference for a break down of what things are. the goal is to not change api/v1 routes and api models after they are defined with `api/v1`.

database layer -> currently in memory in the handlers module
business logic -> handlers module
api logic -> api module
testing -> nonexistant

models -> models module
- [X] core
- [ ] operations

## general structure
- main.rs
- handlers/ in mem data and some business logic
- models/ structs for core
- api/ api logic and response builder

## goal structure
- main.rs
- database/
- handlers/ business logic
- api/
    - models/ api request/response logic
    - routes/ api routes
- models/ structs for core and operations also state machines for flows
- tests/ <- end to end tests

┌─────────────────────────────────────────────────────────┐
│  routes/     │ "How to reach endpoints"                 │
│  api/        │ "What the endpoints accept/return"       │  
│  handlers/   │ "What business logic to execute"         │
│  database/   │ "How to persist/retrieve data"           │
│  models/     │ "What the data looks like"               │
│  tests/      │ "Proof it all works"                     │
└─────────────────────────────────────────────────────────┘

how your mom thinks the project should be laid out
```
src/
├── main.rs
├── routes/
│   ├── mod.rs
│   ├── equipment.rs        // Equipment-related routes
│   ├── modes.rs            // Mode-related routes  
│   └── operations.rs       // Work orders, jobs, etc.
├── api/
│   ├── mod.rs
│   ├── requests/           // Input DTOs
│   ├── responses/          // Output DTOs  
│   └── validation/         // Input validation
├── handlers/
│   ├── mod.rs
│   ├── equipment.rs        // Equipment business logic
│   ├── modes.rs            // Mode business logic
│   └── operations.rs       // Operations business logic
├── database/
│   ├── mod.rs
│   ├── repositories/       // Data access layer
│   ├── migrations/         // Schema changes
│   └── connection.rs       // DB connection handling
├── models/
│   ├── mod.rs
│   ├── core/               // Core domain models
│   ├── operations/         // Operations models
│   └── state_machines/     // Workflow state machines
└── tests/
    ├── integration/        // End-to-end tests
    ├── unit/               // Unit tests
    └── fixtures/           // Test data
```

## sql databases, schemas, stored procedures and functions
these are located in the /sql file

## testing
test for specific functions or implementations should be located in the file with said functions or implementations. 

## future goal
- gRPC support in addition to RestAPI