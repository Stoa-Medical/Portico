# Project Vision

Portico's mission is to **minimise the distance between an idea and a working automation**.

In practice that means:

1. **First-class Agent Model** – developers define *what* should happen as clear, typed Steps; Portico takes care of *how* and *where* it runs.
2. **Data-centric Approach** – every action is traceable because Signals, RuntimeSessions, and Agent state live in a relational database (Postgres).
3. **Human-friendly UI** – a desktop/web app that lets non-engineers configure and monitor automations without touching YAML or a terminal.

### MVP Success Criteria

* CRUD for Agents, Steps, and Signals via the UI or API.
* A Signal can trigger an Agent run and persist a RuntimeSession.
* A non-developer can download the desktop app, configure an Agent, run it, and see a result.

Beyond the MVP, Portico aims to evolve into an extensible integration platform (think *Zapier + LangChain* in OSS form) where community-authored Agent modules can be shared and composed.
