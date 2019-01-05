
# candouble

[![Build Status](https://api.travis-ci.org/thoughtworks/candouble.svg?branch=master)](https://travis-ci.org/thoughtworks/candouble)

Conceptually candouble is very similar to Mountebank and we've tried to use 
the same terminoloy. It might be worthwhile to read the following pages on 
the Mountebank site:

* [Mental Model](http://www.mbtest.org/docs/mentalModel)
* [Stub](http://www.mbtest.org/docs/api/stubs)

At the moment candouble can only load imposters from files. Like Mountebank it
will get a REST interface to define stubs and verify interactions.


## Stub definition

A stub has a list of predicates and a list of responses, e.g.

    {
        "predicates": [
            { "eq": { "id": "0x0101" } 
        ],
        "responses": [
            { "id": "0x01", "data": [ "0x17" ] },
            { "id": "0x01", "data": [ "0x17", "0x20" ] }
        ]
    }
   
When all predicates match an incoming message one response is sent. If 
multiple responses are defined, candouble will cycle through the list of
responses. 
 
In the example, when a message is received that has `0x101` as its id then
candouble will respond with a message with id `0x01` and `0x17` as data. 
(Given that this is CAN, a response can be up to 8 bytes long.) When a 
second message matching the predicate is received candouble will send the
second response, a message with `0x17, 0x20` as data bytes.
 
 
## Stub evaluation
 
The stubs are evaluated in the sequence they are defined in. The first
stub that has a matching predicate will generate the response.
 
 
## Predicates
 
Currently, candouble supports two predicate types only. (Therefore it's
currently not too useful to define multiple predicates for one stub.)
 
     { "eq": { "id": "0x0101" }
     { "msg": { "id": "0x0101", "data": ["*", "0x02"] } 
 
The `eq` type makes it possible to match on message id. The `msg` type 
allows to match on the id and data bytes. An asterisk can be used to 
match any value.
 
 
## Responses
 
Responses are sent as defined. A `_behaviors` attribute can be added to 
the response definition. It is not sent but defines how the stub will send 
the response, e.g.

    { "id": "0x01", "data": [ "0x17" ], "_behaviors": [ { "wait": 50 } ] }

In this case the stub will wait for 50ms before sending the response.

Other possible behaviours are `repeat`, `drop`, and `concat`. They will
be described soon. For now, please have a look at the unit tests in 
`stub.rs`.


## CAN hardware adaptors

If you're on a Mac and have the PCAN adaptor attached, you should run the
application with the `pcan` feature. For it to find the native library you
have to set the dynamic library loading path:

    export LD_LIBRARY_PATH=./lib/PCBUSB
    cargo run --features pcan tests/it_imposter.json
    
If you're not on a Mac then you can run the unit tests, but there are no 
adaptors yet for CAN hardware.


## Web API Examples

    curl -i -X POST -H 'Content-Type: application/json' http://localhost:8080/imposters --data '{ "id": 1, "stubs": [ { "predicates": [{ "eq": { "id": "0x200" } }], "responses": [{ "id": "0x201", "data": [ "0x01" ] }] } ] }'
    
    curl -i http://localhost:8080/imposters
    
    curl -i http://localhost:8080/imposters/1
    
