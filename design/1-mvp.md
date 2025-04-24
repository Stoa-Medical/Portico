*Portico Server* – Design Doc (Dev)

Last significant update: Apr 21, 2025

Overview:

* Data isn’t organized well, and there’s too much vaporware adding complexity instead of removing it. Portico is an agentic integration engine (aka interface engine) for connecting and mapping data with Python code and LLM prompts.
  * By simplifying and consolidating the **integration layer**, this will free-up developers and enable them to build tools that actually service providers.
* There are 2 main services: the **server** and the end-user **app for configuration**
  * **server** – this is backend service with 3 components:
    * **Rust runtime service**: runtime for agent code
    * **Supabase**: handles auth \+ database (***source of truth***)
    * **Python bridge service:** listens to changes in Supabase and forwards them to Rust service
  * **app** – this is a Tauri desktop app that can update servers / run in demo mode

Goals (min 1, max 3):

1. Prototype to prove traction (pitch to various people)
2. CRUD interface and simple interface
3. Balance scalability and hacking

Non-Goals (min 1, max 3):

1. No horizontal scaling thinking – just get the first thing going
2. No crazy UX development – this thing gonna look *simple*
3. No wacky local LLM dev. That’d be great one-day, today’s not the day. Together.ai is perfect at this stage

Milestones \+ Timelines (min 1\):

1. 2025-4-30: Server code done, proud to publish and publicize

Key Technical Decisions (min 1):

* **Building on Supabase:** handling auth, realtime, reliability of postgres
  * Using **Supabase** **Realtime** to trigger events to the backend
  * For reading \+ writing data: use plain SQL queries to guarantee ACID compliance. Use SQLx Rust lib to keep it close to SQL
* **Building in Rust \+ PyO3 instead of pure Python**: handling scalability and also keeps code thinking consistent between server+app (e.g. keeping structs in \`common\` lib)
* **Use threadpool and wait for Python 3.14 for non-GIL approach**: Live with Python GIL restriction, see if it fixes itself. Looked into multiprocessing but management was just too much work (though this is a viable approach since the size of a Python interpreter session is \~50mb, not big)

Key Data Models:

* **Signal:** Something that happens and can trigger actions.
  * A Signal is the core “event” in this event-based architecture. It can either come with some data that needs to be run, some data that’s stashed as FYI, or just an event as an FYI.
  * A Signal can request an Agent to do something and wait for a result – the result of the run is stored back to the Signal.
  * There are three types of Signals:
    * sync: Requests that the engine state pulls from the database
    * command: Triggers a specific action/state change (e.g. Agent update, Step delete, Agent run, Step run, etc.)
    * fyi: Doesn’t trigger anything, used for logging data with timestamp
* **Agent**: this represents an “automation” unit. It does things with Steps and either starts or responds to Signals.
* **Step**: a unit of action that an Agent can take. It's either deterministic (code – Python) or non-deterministic (calling an LLM).
  * Each Step expects a JSON Value as input, and returns a Result\<Value, Error\> as output.
  * A step is like a function – it defines what should be done, though the instantiation/execution happens at runtime.
* **RuntimeSession:** the execution environment of an Agent applying Steps to some data.
  * This is created by an Agent and represents the execution environment \+ details associated with corresponding Steps.
  * This model both tracks

Key Interactions:

* **Signal:**
  * Creation triggers other actions.
* **Agent**:
  * Contains Steps and a described goal
  * Starts RuntimeSession(s)
* **Step**:
  * Owned by Agents. Each Agent creates its own copies of Steps (Steps *are not* shared between Agents)
  * Configured to either 1\) execute Python code, or 2\) call an LLM
    * Assumes JSON input and output
* **RuntimeSession**:
  * Started by an Agent and represents an individual run with data against the Agent’s Steps
  * Saves information on runtime duration and success of different steps

**Key Workflow (server)**:

1. User-specified config in \`.env\` variable
2. Supabase services start
3. Rust service starts and connects to Supabase
   1. Rust container establishes gRPC server which Python server connects to
   2. Rust container queries Supabase for initial state
4. Python service starts and connects to Rust gRPC server \+ Supabase
   1. Sends init message
   2. Forwards events via Supabase Realtime
5. Signals come-into Supabase (triggered by the UI, or by connecting directly to the Supabase PostgREST API)
   1. Signals get forwarded to the Bridge service via Supabase Realtime
   2. The Bridge service identifies what kind of Signal action should be taken and forwards it to the Rust service as a Protobuf message.
   3. The Rust service puts the Protobuf message in a central message queue which gets processed in-order. Each Agent has its own message queue which feeds from the central queue. Agents process messages in-order on available threads (each Agent can spawn its own thread or use one from the available threadpool).
   4. The Agent processes data and generates a RuntimeSession that denotes how long execution took via timestamps and other data.
   5. The RuntimeSession is saved to the Supabase database.
   6. The Signal is updated in the Supabase database with the result and RuntimeSession linked.

Concluding notes:

Using the above workflow, users can:

* Trigger Signals in the server and have pass data to an Agent, and the Agent can execute Steps and return the result
* Connect services to the Signals API to trigger actions. Coming from the “traditional” API
* Calculate analytics using RuntimeSession table

This constitutes the “core” features of the open-core version of Portico. Advanced features that I’ve thought about, though are not scoped here, include (and are not limited to):

* AI Customization
* Metrics via OpenTelemetry and Graphana
* Pre-configured Agent modules
* Advanced analytics
* Pre-configured ETLs and consolidation of different features
* MCP Server implementation and support
* FHIR Server implementation and support
