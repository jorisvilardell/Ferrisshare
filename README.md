# Polytech DevOps - System Programming Project

## Rules

Students MUST pick one out of the [four proposed topics](topics).

Before the project deadline, students MUST turn in an [architecture document](#architecture-and-documentation) and an [implementation](#implementation) for the selected topic, in the form of a [GitHub pull request](#project-contribution).

## Topics

topics choosed:

1. [A peer-to-peer file transfer protocol](topics/p2p-transfer-protocol).

## Grade

Your work will be evaluated based on the following criteria:

### Architecture and Documentation

**This accounts for 40% of the final grade**

You MUST provide a short, `markdown` formatted and English written document describing your project architecture.

It MUST live under a projects top level folder called `docs/`, e.g. `docs/architecture.md`.

It SHOULD at least contain the following sections:

1. Project definition: What is it? What are the goals of the tool/project?
1. Components and modules: Describe which modules compose your project, and how they interact together. Briefly justify why you architectured it this way.
1. Usage: How can one use it? Give usage examples.

### Implementation

**This accounts for 40% of the final grade**

The project MUST be implemented in Rust.

The implementation MUST be formatted, build without warnings (including `clippy` warnings) and commented.

The implementation modules and crates MAY be unit tested.

### Project Contribution

**This accounts for 20% of the final grade**

The project MUST be submitted as one single GitHub pull request (PR) against the [current](https://github.com/dev-sys-do/project-2427) repository, for the selected project.

For example, a student picking the `p2p-transfer-protocol` topic MUST send a PR that adds all deliverables (source code, documentation) to the `topics/p2p-transfer-protocol/` folder.

All submitted PRs will not be evaluated until the project deadline. They can thus be incomplete, rebased, closed, and modified until the project deadline.

A pull request quality is evaluated on the following criteria:

- Commit messages: Each git commit message should provide a clear description and explanation of what the corresponding change brings and does.
- History: The pull request git history MUST be linear (no merge points) and SHOULD represent the narrative of the underlying work. It is a representation of the author's logical work in putting the implementation together.

A very good reference on the topic: https://github.blog/developer-skills/github/write-better-commits-build-better-projects/

### Grade Factor

All proposed topics have a grade factor, describing their relative complexity.

The final grade is normalized against the selected topic's grade factor: `final_grade = grade * topic_grade_factor`.

For example, a grade of `8/10` for a topic which grade factor is `1.1` will generate a final grade of `8.8/10`.

## Deadline

All submitted PRs will be evaluated on October 30th, 2025 at 11:00 PM UTC.

# A P2P Transfer Protocol

## Description and Goal

Build a CLI tool that allows two users on the same network to transfer a single file to each other.
The tool should be able to act as both the sender and the receiver, without a central server.

It is expected for a sender to know the IP of the receiver, i.e. there is no discovery protocol.

```shell
# Receiving a file on port 9000
p2p-tool listen --port 9000 --output ./shared

# Sending a file
p2p-tool send --file report.pdf --to 192.168.1.100 --port 9000
```

## Hints and Suggestions

- Define and document a simple networking protocol with a few commands. For example
  - HELLO: For the sender to offer a file to the receiver. It takes a file size argument.
  - ACK: For the receiver to tell the sender it is ready to receive a proposed file.
  - NACK: For the receiver to reject a proposed file.
  - SEND: Send, for the sender to actually send a file. It also takes a file size argument, that must match the `HELLO` offer.
- Start a receiving thread for every sender connection.

## Grade Factor

The grade factor for this project is _1_.
