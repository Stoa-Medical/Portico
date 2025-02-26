Portico MVP – Design Doc (Dev)

Last significant update: Feb 19, 2025

Overview:

- Data isn’t organized well, and there’s too much vaporware adding complexity instead of removing it.
  - The “EHR” should be separately a database and a frontend. And people don’t want to build their own UIs – they want a good UI out-of-the-box (... this comes later).
- This will free-up developers and enable them to build tools that actually service providers.
- Ok there are 2 main services: the server and the app
  - server – this is backend service with 2 components:
    - Rust server event loop: schedules Agent and runs the code
      - Based on Events that happen in the data
    - Supabase (Postgres): handles auth + database (source of truth)
  - app – this is a Tauri desktop app that can update servers / run in demo mode. It can do the following things:
    - Connect to a server: authenticates properly w/ Supabase best practice
    - UI for CRUD operations: connects to a server and configures the things
    - “Demo mode”: guest access that runs a local session and python interpreter

Goals (min 1, max 3):

1. Prototype to prove traction (pitch to various people)
2. CRUD interface and simple interface
3. Balance scalability and hacking

Non-Goals (min 1, max 3):

1. No horizontal scaling thinking – just get the first thing going
2. No crazy UX development – this thing gonna look simple
3. No wacky local LLM dev. That’d be great one-day, today’s not the day. Together.ai is perfect at this stage

Key Technical Decisions (min 1):

- Building on Supabase: handling auth, realtime, reliability of postgres
  - Using Supabase Realtime to trigger events (this encapsulates the pubsub functionality)
  - For reading + writing data: use plain SQL queries to guarantee ACID compliance. Keep it close to SQL – use SQLx Rust lib
- Building with Tauri: security guarantees / improvements
- Building with Svelte: Prefer Svelte or Vanilla JS
- Building in Rust + PyO3 instead of pure Python: handling scalability and also keeps code thinking consistent between server+app (e.g. keeping structs in `common` lib)
- Leaving data manipulation to the Python layer: bank on pydian being the “right” way to express data mappings. Performance negligible for this kind of stuff (handling big stuff / multithreading already with Rust, can revisit if we get there)

Key Data Models:

- Signal: Something that happens and can trigger actions.
  - A Signal is the core “event” in this event-based architecture
  - A Signal comes with some data
  - A Signal has information about what happened
  - The name “Event” is not used to avoid confusion with the OpenTelemetry MELT naming convention (this isn’t used here, though will be used later)
    - A Signal corresponds to a “Trace” in Otel abstractions
- Schedule: Triggers a Signal based on a CRON schedule
- Channel: A persistent listener (traditional interface engine concept). Triggers a Signal based on the following possible areas:
  - HTTPS endpoints
  - FTP/SFTP endpoints (+ local file system)
  - MLLP for HL7v2 connections (an extension of TCPIP buffers)
  - SOAP + XML endpoints
- Mission: a request to some Agent(s) to do some task. Might include some data to run.
  - This is created from a Signal
  - The actual task execution is abstracted-away to the Agent(s)
- Agent: this represents an “AI Agent”. It does things with Steps and either starts or responds to Signals.
- Step: a unit of action that an Agent can take. It's either deterministic (Python code) or non-deterministic (calling an LLM).
  - Each Step expects a JSON Value as input, and returns a Result<Value, Error> as output.
  - A step is like a function – it defines what should be done, though the instantiation/execution happens at runtime
- RuntimeSession: the execution environment of an Agent applying Steps to some data.
  - This is created by an Agent and represents the execution environment + details associated with corresponding Steps

Key Interactions:

- Signal:
  - Creates Mission. Tracks the status and completes based on Mission result
- Mission:
  - Specifies some Agent(s) to perform some task
- Agent:
  - Contains Steps and a described goal
  - Starts RuntimeSession(s)
- Step:
  - Owned by Agents. Each Agent creates its own copies of Steps (Steps are not shared between Agents)
  - Configured to either 1) execute Python code, or 2) call an LLM
    - Assumes JSON input and output
- RuntimeSession:
  - Started by an Agent
  - Each RuntimeSession lives on a thread spawned by the main event threadpool

Key Workflow (server):

1. User-specified config in `.env` variable and Dockerfiles
2. Supabase starts in container
3. Rust container starts (runs socketioxide)
4. Rust container queries Supabase for initial state
5. Rust container polls for changes in Supabase data model
   1. Have a WebSocket listening to `Signal` via Supabase Realtime features
   2. Respond to Signals to do simple CRUD and initiate RuntimeSessions

Proposed Dev Architecture: Portico-MVP_4-Dev-Architecture (Dev-like)

Proposed UI Views: <TODO – link to PenPot>
