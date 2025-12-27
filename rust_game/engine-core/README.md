# Overview

This crate is an engine for building real-time multiplayer games.  It provides
traits whose implementations define [client side input
aggregation][AggregateInput], [deterministic game update logic][Play], and
[correction smoothing][Smooth] functions.  Once defined, the engine runs the
deterministic logic on a fixed interval on both an authoritative server and
local clients.  Local clients perform client-side prediction using local inputs
as well as rollback to incorporate inputs from remote clients and authoritative
state snapshots from the server.  Client's rollback is immediately followed by a
fast-forward re-run of the game logic to correct the current state.

This crate provides a channel-like interface to the local client.  There is a
[sender][Client] struct that is used to send inputs to the client, and a
[receive][SmoothReceiver] struct that is used to receive a current, smoothed
state of the game.  A similar sender/receiver pair of structs provide an
interface for the server.

This crate implements all of the necessary networking code to enable the clients
and server to communicate and synchronize the state of the game.

## What this crate is not

This crate is not and does not implement any components of:

* A graphics library
* An Entity-Component-System library
* A game I/O library

# Key Concepts

## State
A [State][Play::State] is the State of the game at a specific frame.  The game
advances through time by using [game update logic][Play::next_state] to compute
the next State.

The current State is one of the arguments to the function which computes the
next State.

## Frame

A Frame is the State of the game and client's Input Aggregates at a specific
point in time.  The engine runs the game's update logic on a fixed interval or
timestep.  Each Frame corresponds precisely to one member in the game's sequence
of updates.  

The amout of time between when two sequential Frames theoretically occur is
fixed and does not vary from Frame to Frame.

The State of the game only exists discretely at a Frame.  There is no concept of
a State existing at a continuous time between Frames.

Similar to States, Input Aggregates only exist discretely at a Frame.

Both the server and clients keep a sequence of recent frames which are used to
implement prediction, rollback, and fast-forward as local and remote Input
Aggregates and authoritative State snapshots are received.

A Frame has no relationship to frames as rendered on a display by a graphics
library.  This crate uses [smoothing][Smooth] to decouple Frames of the game's
update logic and frames rendered on a display.

## Input Event

An [Input Event][Play::InputEvent] is an event that can be sent to the client
game at any continuous point in time (i.e. does not need to occur at a Frame).
The client aggregates Input Events that it receives over the course of a Frame
into an Input Aggregates.

Input Events provide a way to decouple the game's update logic an Frames from
the polling or reception of player's I/O.

Input Events are not arguments to the [function][Play::next_state] which
computes the next State.

## Input Aggregate

An [Input Aggregate][Play::InputAggregate] is a client's input at a specific
Frame.  Input Aggregates are formed by aggregating Input Events using a function
defined by [AggregateInput].

Over the course of a single Frame, local clients will produce a new Input
Aggregate for that Frame after each Input Event is received.  This Input
Aggregate is used to predict the future State of the game, even though it may be
superseded by another Input Aggregate that is produced if another Input Event is
received.  This is done to allow the local game to be as responsive as possible
to the local user's input.  The final Input Aggregate that the client produces
for the Frame will be the one the server uses for the authoritative update of
the game.

All remote client's Input Aggregates are arguments to the
[function][Play::next_state] which computes the next State.

## Authoritative and Non-Authoritative

Both Input Aggregates and States can be Authoritative or Non-Authoritative.

States are Authoritative if they were computed from a previous Authoritative
State and a set of Input Aggregates that are Authoritative for all clients.
Otherwise, the State is Non-Authoritative.

Input Aggregates are Authoritative when the client sends the final Input
Aggregate created for a Frame to the server and the server accepts it and
redistributes it back to the clients.  Otherwise, the Input Aggregate is
Non-Authoritative.

Clients always add their local Input Aggregates to the game as
Non-Authoritative.  This is done because clients do not know that their Input
Aggregate will be received by the server in a timely manner.  There is a chance
that it could be dropped by the network or rejected by the server as being late.

The authoritativeness of States and Input Aggregates is exposed to the game
update [function][Play::next_state] through the [FrameState] and [FrameInput]
enums.

## Smoothing

Smoothing helps smooth changes to the State of the game.  Smoothing is a single operation that serves two purposes.  The first is to hide jerks or jumps from abrupt error corrections to the State.  The second is interpolating between sequential States to provide the illusion of continuity of the game between descrete Frames.

Smoothing can be thought of as an interpolation/extrapolation between the last [SmoothedState][Smooth::SmoothedState] (this can be thought of as the last state that was rendered on the display, even though its not technically that) and a future State.  The future State is typically a predicted, Non-Authoritative State.

# Getting Started

* Implement your game's update logic by implementing the [Play] trait.  This includes
    * Define a [Play::State] type
    * Define a [Play::InputEvent] type
    * Define a [Play::InputAggregate] type
    * Define a [Play::InputAggregator] and implement [AggregateInput]
* Implement [Smooth] for your [Play] implementation

