# Documentation Index

Complete guide to PhantomRelay documentation.

---

## Quick Start

**New to PhantomRelay?** Start here:
1. Read [README.md](./README.md) - 10 min overview
2. Skim [ARCHITECTURE.md](./ARCHITECTURE.md) - High-level design
3. Try [DEPLOYMENT.md](./DEPLOYMENT.md) - Get it running

---

## Documentation Files

### Core Documentation

| File | Purpose | Audience | Read Time |
|------|---------|----------|-----------|
| [README.md](./README.md) | Project overview, features, building, running | Everyone | 10 min |
| [ARCHITECTURE.md](./ARCHITECTURE.md) | High-level system design, component overview, data flows | Developers, Architects | 20 min |
| [COMPONENTS.md](./COMPONENTS.md) | Deep dive into each component, specifications, algorithms | Developers working on specific subsystems | 30 min |
| [DATA_FLOWS.md](./DATA_FLOWS.md) | Complete request paths, state machines, concurrency patterns | Developers debugging or optimizing | 25 min |
| [DEVELOPER_GUIDE.md](./DEVELOPER_GUIDE.md) | Project structure, patterns, testing, common tasks | Contributors, maintainers | 20 min |
| [DEPLOYMENT.md](./DEPLOYMENT.md) | Installation, configuration, monitoring, troubleshooting, scaling | Ops, DevOps engineers | 30 min |

---

## How to Use This Documentation

### I want to...

**...understand what PhantomRelay does**
- → Read [README.md](./README.md) Overview section

**...understand how it works architecturally**
- → Read [ARCHITECTURE.md](./ARCHITECTURE.md)

**...set up PhantomRelay**
- → Follow [DEPLOYMENT.md](./DEPLOYMENT.md) Building & Installation sections

**...start and control the daemon**
- → See [README.md](./README.md) Running section
- → Reference [DEPLOYMENT.md](./DEPLOYMENT.md) Startup section

**...understand a specific component (DNS, Proxy, TProxy, etc.)**
- → Check [COMPONENTS.md](./COMPONENTS.md) for that component
- → Cross-reference [DATA_FLOWS.md](./DATA_FLOWS.md) for request flow through it

**...add a new feature**
- → Read [DEVELOPER_GUIDE.md](./DEVELOPER_GUIDE.md) Adding New Functionality
- → Reference [COMPONENTS.md](./COMPONENTS.md) for component specs

**...fix a bug**
- → Follow the issue's data flow in [DATA_FLOWS.md](./DATA_FLOWS.md)
- → Check relevant component in [COMPONENTS.md](./COMPONENTS.md)
- → Use patterns from [DEVELOPER_GUIDE.md](./DEVELOPER_GUIDE.md) Common Patterns

**...monitor or troubleshoot**
- → See [DEPLOYMENT.md](./DEPLOYMENT.md) Monitoring & Troubleshooting sections

**...debug a concurrency issue**
- → Study [DATA_FLOWS.md](./DATA_FLOWS.md) Concurrency Scenarios
- → Review [COMPONENTS.md](./COMPONENTS.md) Concurrency Model

**...understand the CLI tool**
- → [README.md](./README.md) Running section
- → [DEVELOPER_GUIDE.md](./DEVELOPER_GUIDE.md) Project Layout - cli/

**...optimize performance**
- → [DEPLOYMENT.md](./DEPLOYMENT.md) Performance Tuning
- → [DEVELOPER_GUIDE.md](./DEVELOPER_GUIDE.md) Performance Debugging

**...scale PhantomRelay**
- → [DEPLOYMENT.md](./DEPLOYMENT.md) Scaling Considerations

---

## Documentation Architecture

### Layer 1: User-Facing (Overview)
```
README.md
├─ Features
├─ Building & Running
├─ CLI Commands
└─ Quick Start
```

### Layer 2: System Design (Architecture)
```
ARCHITECTURE.md
├─ High-level overview
├─ Core components
├─ Data flow patterns
└─ Dependency graph
```

### Layer 3: Implementation Detail (Components)
```
COMPONENTS.md
├─ Component specifications
├─ Algorithms & Data structures
├─ Request flows
└─ State machines
```

### Layer 4: Operational (Data Flows)
```
DATA_FLOWS.md
├─ Complete request paths
├─ Concurrency scenarios
├─ State transitions
└─ Error recovery
```

### Layer 5: Developer (Practical)
```
DEVELOPER_GUIDE.md
├─ Project structure
├─ Code patterns
├─ Testing & debugging
└─ Contributing guidelines
```

### Layer 6: Operations (Deployment)
```
DEPLOYMENT.md
├─ Installation methods
├─ Configuration
├─ Monitoring & troubleshooting
├─ Performance tuning
└─ Scaling
```

---

## Key Concepts Across Docs

### Service-Oriented Architecture
- **Explained in**: [ARCHITECTURE.md](./ARCHITECTURE.md) Runtime Layer
- **Components**: [COMPONENTS.md](./COMPONENTS.md) Runtime & Service Management
- **Operations**: [DEPLOYMENT.md](./DEPLOYMENT.md) Startup & Verification

### Event Bus Pattern
- **Explained in**: [ARCHITECTURE.md](./ARCHITECTURE.md) Monitoring & Event System
- **Details**: [COMPONENTS.md](./COMPONENTS.md) Monitoring & Events
- **Usage**: [DATA_FLOWS.md](./DATA_FLOWS.md) Event patterns throughout

### Concurrent Data Structures
- **Overview**: [ARCHITECTURE.md](./ARCHITECTURE.md) Concurrency Model
- **Deep dive**: [COMPONENTS.md](./COMPONENTS.md) Concurrency Model
- **Examples**: [DATA_FLOWS.md](./DATA_FLOWS.md) Concurrency Scenarios

### Proxy Rotation
- **Feature**: [README.md](./README.md) Intelligent Proxy Rotation
- **Design**: [ARCHITECTURE.md](./ARCHITECTURE.md) Proxy Rotation Engine
- **Implementation**: [COMPONENTS.md](./COMPONENTS.md) Proxy Rotation Engine
- **Flow**: [DATA_FLOWS.md](./DATA_FLOWS.md) Path 3: Proxy Rotation Service
- **Configuration**: [DEPLOYMENT.md](./DEPLOYMENT.md) Configuration

### DNS Caching
- **Feature**: [README.md](./README.md) Advanced DNS Resolution
- **Design**: [ARCHITECTURE.md](./ARCHITECTURE.md) DNS Resolution Subsystem
- **Implementation**: [COMPONENTS.md](./COMPONENTS.md) DNS Subsystem
- **Flow**: [DATA_FLOWS.md](./DATA_FLOWS.md) Path 2: DNS Query Resolution
- **Tuning**: [DEPLOYMENT.md](./DEPLOYMENT.md) Performance Tuning

### Transparent Proxy
- **Feature**: [README.md](./README.md) Transparent Proxy (TProxy)
- **Design**: [ARCHITECTURE.md](./ARCHITECTURE.md) Transparent Proxy (TProxy)
- **Implementation**: [COMPONENTS.md](./COMPONENTS.md) Transparent Proxy (TProxy)
- **Flow**: [DATA_FLOWS.md](./DATA_FLOWS.md) Path 1: System Traffic Through TProxy
- **Setup**: [DEPLOYMENT.md](./DEPLOYMENT.md) Network Configuration

---

## Learning Paths

### For New Users
1. [README.md](./README.md) - Understand what it does
2. [DEPLOYMENT.md](./DEPLOYMENT.md) - Build and run it
3. [README.md](./README.md) Running section - Try CLI commands

### For Operators/DevOps
1. [README.md](./README.md) Features section
2. [DEPLOYMENT.md](./DEPLOYMENT.md) Complete (Installation through Troubleshooting)
3. [DEVELOPER_GUIDE.md](./DEVELOPER_GUIDE.md) - Common Issues section

### For Backend Developers
1. [README.md](./README.md) - Context
2. [ARCHITECTURE.md](./ARCHITECTURE.md) - System overview
3. [COMPONENTS.md](./COMPONENTS.md) - Pick components to work on
4. [DEVELOPER_GUIDE.md](./DEVELOPER_GUIDE.md) - Project structure and patterns

### For Performance Engineers
1. [ARCHITECTURE.md](./ARCHITECTURE.md) Concurrency Model
2. [COMPONENTS.md](./COMPONENTS.md) Concurrency Model
3. [DATA_FLOWS.md](./DATA_FLOWS.md) Concurrency Scenarios
4. [DEPLOYMENT.md](./DEPLOYMENT.md) Performance Tuning & Scaling
5. [DEVELOPER_GUIDE.md](./DEVELOPER_GUIDE.md) Performance Debugging

### For Architects/Designers
1. [ARCHITECTURE.md](./ARCHITECTURE.md) Complete
2. [COMPONENTS.md](./COMPONENTS.md) Complete
3. [DATA_FLOWS.md](./DATA_FLOWS.md) Data Flow Patterns

---

## Common Tasks & Where to Find Help

| Task | Primary Doc | Secondary Docs |
|------|------------|-----------------|
| Build from source | [DEPLOYMENT.md](./DEPLOYMENT.md) Building | [README.md](./README.md) Building |
| Install as systemd service | [DEPLOYMENT.md](./DEPLOYMENT.md) Installation | - |
| Configure DNS | [DEPLOYMENT.md](./DEPLOYMENT.md) Configuration | [COMPONENTS.md](./COMPONENTS.md) DNS |
| Setup TProxy interception | [DEPLOYMENT.md](./DEPLOYMENT.md) Network Configuration | [COMPONENTS.md](./COMPONENTS.md) TProxy |
| Add custom proxy source | [DEVELOPER_GUIDE.md](./DEVELOPER_GUIDE.md) Adding Configuration | [COMPONENTS.md](./COMPONENTS.md) Rotation |
| Debug connection issues | [DEPLOYMENT.md](./DEPLOYMENT.md) Troubleshooting | [DATA_FLOWS.md](./DATA_FLOWS.md) Error Recovery |
| Optimize for high throughput | [DEPLOYMENT.md](./DEPLOYMENT.md) Performance Tuning | [COMPONENTS.md](./COMPONENTS.md) Performance |
| Add logging to component | [DEVELOPER_GUIDE.md](./DEVELOPER_GUIDE.md) Common Patterns | [COMPONENTS.md](./COMPONENTS.md) Monitoring |
| Create new service | [DEVELOPER_GUIDE.md](./DEVELOPER_GUIDE.md) Adding New Service | [COMPONENTS.md](./COMPONENTS.md) Service types |
| Understand DNS cache flow | [DATA_FLOWS.md](./DATA_FLOWS.md) Path 2 | [COMPONENTS.md](./COMPONENTS.md) DNS |
| Monitor in production | [DEPLOYMENT.md](./DEPLOYMENT.md) Monitoring & Observability | - |

---

## Document Dependencies

```
README.md (standalone introduction)
    ↓
ARCHITECTURE.md (high-level understanding)
    ├─ COMPONENTS.md (detailed specs)
    │   ├─ DATA_FLOWS.md (operational details)
    │   └─ DEVELOPER_GUIDE.md (implementation)
    │
    └─ DEPLOYMENT.md (practical operations)
        └─ DEVELOPER_GUIDE.md (troubleshooting)
```

**Read order recommendation**: 
1. Start with README.md
2. Choose your path based on your role above
3. Deep dive into specific components as needed

---

## Documentation Standards

All documents follow these standards:

- **Clear structure**: Headers, sections, subsections
- **Code examples**: Practical, runnable examples where applicable
- **Diagrams**: ASCII diagrams for complex flows
- **Tables**: Organized reference information
- **Cross-references**: Links between related concepts
- **Plain language**: Technical but accessible
- **Context**: Each section self-contained but linked

---

## Contributing to Documentation

To add or update documentation:

1. **Identify the right file** based on content type:
   - Feature description → README.md
   - Architecture pattern → ARCHITECTURE.md
   - Component deep dive → COMPONENTS.md
   - Example or flow → DATA_FLOWS.md
   - Developer guide → DEVELOPER_GUIDE.md
   - Deployment/ops → DEPLOYMENT.md

2. **Maintain structure**: Use consistent heading levels and formatting

3. **Add cross-references**: Link to related sections in other docs

4. **Include examples**: Practical code or command examples

5. **Update this index** if creating new sections or files

---

## Version History

| Version | Date | Changes | Docs |
|---------|------|---------|------|
| 0.1.11 | 2024-05-25 | Initial comprehensive documentation suite | All |
| 0.1.10 | 2024-05-20 | Rotation engine refinement | ARCHITECTURE, COMPONENTS |
| 0.1.9 | 2024-05-15 | DNS prewarmer implementation | README |

---

## Questions & Feedback

- **How do I...?** → Check the Learning Paths above
- **I found an issue** → Check Troubleshooting in [DEPLOYMENT.md](./DEPLOYMENT.md)
- **Documentation unclear** → File an issue with doc location and section
- **Missing documentation** → Check issue tracker or file enhancement request

