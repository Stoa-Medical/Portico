# Portico Refactor

Last significant update: January 27, 2026

## Overview

The original MVP for Portico was over-indexed on systems engineering when the actual stage is scrappier prototype capturing core ideas.

The goal of this refactor is to focus the scope 

## Key User Goals 
1. Connect an HL7v2 feed from an EHR via TCP or MLLP
2. Have AI organize the data visually
3. Type out workflows 

### Key User Workflows:
1. Creating a workflow: log-in, provide example data, 

## Goals (min 1, max 3):
1. Take one HL7v2 message and convert to FHIR using an AI-generated pipeline
    - Start with an ADT-01

## Non-Goals (min 1, max 3):
1. Focus on making one workflow excellent. Do not abstract to other workflows before this.

## Milestones + Timelines (min 1):
1. 

## Key Technical Decisions (min 1):
1. Keep Tauri, switch to React for frontend
2. Focus main server in Python (this was the "bridge" previously)
3. Make sandbox environment in Rust with PyO3

## Key Data Models:
* Signal: 
* Agent: 
* 

### Key Interactions:
* 

## Concluding notes:

The last iteration was too theoretical and didn't meet people where they're at. 
