_Portico Server_ – Design Doc (Dev)

Last significant update: Mar 26, 2025

Overview:

- Data isn’t organized well, and there’s too much vaporware adding complexity instead of removing it.
  - The “EHR” should be separately a database and a frontend. And people don’t want to build their own UIs – they want a good UI out-of-the-box (... this comes later).
- This will free-up developers and enable them to build tools that actually service providers.
- Ok there are 2 main services: the **server** and the **app.**
  - **server** – this is backend service with 3 components:
    - **Rust runtime service**: schedules Agent and runs the code
    - **Supabase**: handles auth \+ database (**_source of truth_**)
    - **Python bridge service:** listens to changes in Supabase and forwards them to Rust service
  - **app** – this is a Tauri desktop app that can update servers / run in demo mode. For more details, see [Portico-App_3-Design-Doc (Dev-like)](https://docs.google.com/document/d/1EkWZnTxHprff4JHlzr9l9Rq6ef42W50_imTPW0NvE3Y/edit?tab=t.0)

Goals (min 1, max 3):

1. Prototype to prove traction (pitch to various people)
2. CRUD interface and simple interface
3. Balance scalability and hacking

Non-Goals (min 1, max 3):

1. No horizontal scaling thinking – just get the first thing going
2. No crazy UX development – this thing gonna look _simple_
3. No wacky local LLM dev. That’d be great one-day, today’s not the day. Together.ai is perfect at this stage

Milestones \+ Timelines (min 1\):

1. 2025-03-31: Server code done, proud to publish and publicize

—

Key Technical Decisions (min 1):

- **Building on Supabase:** handling auth, realtime, reliability of postgres
  - Using **Supabase** **Realtime** to trigger events (this encapsulates the pubsub functionality)
  - For reading \+ writing data: use plain SQL queries to guarantee ACID compliance. Use SQLx Rust lib to keep it close to SQL
- **Building in Rust \+ PyO3 instead of pure Python**: handling scalability and also keeps code thinking consistent between server+app (e.g. keeping structs in \`common\` lib)
- **Use process pool over threadpool**: Avoid the Python GIL conflict by running an independent Python interpreter for multiple processes (this is fine since no IPC needs to happen – each thread runs independently, re-combined later). The size of a Python interpreter session is \~50mb

Key Data Models:

- **Signal:** Something that happens and can trigger actions.
  - A Signal is the core “event” in this event-based architecture
  - A Signal comes with some data
  - A Signal has information about what happened
  - A Signal can request an Agent to do something and wait for a result
- **Agent**: this represents an “automation” unit. It does things with Steps and either starts or responds to Signals.
- **Step**: a unit of action that an Agent can take. It's either deterministic (code – Python) or non-deterministic (calling an LLM).
  - Each Step expects a JSON Value as input, and returns a Result\<Value, Error\> as output.
  - A step is like a function – it defines what should be done, though the instantiation/execution happens at runtime
- **RuntimeSession:** the execution environment of an Agent applying Steps to some data.
  - This is created by an Agent and represents the execution environment \+ details associated with corresponding Steps
- **Schedule**: Triggers a Signal based on a CRON schedule
- **Channel**: A persistent listener (traditional interface engine concept). Triggers a Signal based on the following possible areas:
  - HTTPS endpoints
  - FTP/SFTP endpoints (+ local file system)
  - TCPIP Connections
  - MLLP for HL7v2 connections (an extension of TCPIP buffers)
  - SOAP \+ XML endpoints

Key Interactions:

- **Signal:**
  - Creation conditionally triggers other actions
- **Agent**:
  - Contains Steps and a described goal
  - Starts RuntimeSession(s)
- **Step**:
  - Owned by Agents. Each Agent creates its own copies of Steps (Steps _are not_ shared between Agents)
  - Configured to either 1\) execute Python code, or 2\) call an LLM
    - Assumes JSON input and output
- **RuntimeSession**:
  - Started by an Agent
  - Each RuntimeSession gets spawned on an independent process as managed by the process pool
- **Channel:**
  - Creates a Signal in response to a message
- **Schedule**:
  - Creates a Signal in response to a CRON schedule. Assume timeserver

**Key Workflow (server)**:

1. User-specified config in \`.env\` variable and Dockerfiles
2. Supabase service(s) start
3. Python service starts
4. Rust service starts
   1. Rust container establishes TCP/IP connection with Python service, receives init message as expected
5. Rust container queries Supabase for initial state
6. Rust container polls for changes in Supabase data model
   1. Have a WebSocket listening to \`Signal\` via Supabase Realtime features
   2. Respond to Signals to do simple CRUD and initiate RuntimeSessions

Proposed Dev Architecture: [Portico-MVP_4-Dev-Architecture (Dev-like)](https://docs.google.com/drawings/d/1ETJTJMPHI3zoXfT6VbXxrRzMMdVPe2QM-sLhLxYT7Sg/edit)
