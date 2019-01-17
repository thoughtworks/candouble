
# Candouble

[![Build Status](https://api.travis-ci.org/thoughtworks/candouble.svg?branch=master)](https://travis-ci.org/thoughtworks/candouble)

Conceptually Candouble is very similar to Mountebank and we've tried to use the 
same terminology. It might be worthwhile to read the following pages on the 
Mountebank site:

* [Mental Model](http://www.mbtest.org/docs/mentalModel)
* [Stub](http://www.mbtest.org/docs/api/stubs)

The central concept is a so called _imposter_ that is attached to a CAN port.
The imposter has a list of stubs that specify how the imposter responds to
incoming messages. In addition, the imposter can record incoming messages, and
these are available for inspection via Candouble's Web API.



## Stubs

### Definition

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
   
When all predicates match an incoming message a response is sent. If multiple
responses are defined, Candouble will cycle through the list of responses.

In the example above, when a message is received that has `0x101` as its id then
Candouble will respond with a message with id `0x01` and `0x17` as data. (Given
that this is CAN, a response can be up to 8 bytes long.) When a second message
matching the predicate is received Candouble will send the second response, a
message with `0x17, 0x20` as data bytes.


### Predicates

Currently, Candouble supports two predicate types only. (Therefore it's
currently not too useful to define multiple predicates for one stub.)

     { "eq": { "id": "0x0101" }
     { "msg": { "id": "0x0101", "data": ["*", "0x02"] } 
 
The `eq` type makes it possible to match on message id. The `msg` type allows to
match on the id and data bytes. An asterisk can be used to match any value.


### Responses

Responses are sent as defined. A `_behaviors` attribute can be added to the
response definition. It is not sent but defines how the stub will send the
response, e.g.

    { "id": "0x01", "data": [ "0x17" ], "_behaviors": [ { "wait": 50 } ] }

In this case the stub will wait for 50ms before sending the response.

Other possible behaviours are `repeat`, `drop`, and `concat`. They will be
described soon. For now, please have a look at the unit tests in `stub.rs`.


## Imposters

The concept of an imposter is borrowed from Mountebank. In a nutshell, an
imposter is a collection of stubs that are active for a given CAN port.

**At the moment Candouble only supports one CAN port, and its id is 0.**


### Definition

An imposter specifies the port id and a list of stubs, e.g.

    {
      "id": 0,
      "stubs": [
        {
          "predicates": [
            { "eq": { "id": "0x0101" } },
          ],
          "responses": [
            { "id": "0x0102", "data": ["0xCA", "0xFE"] }
          ]
        }
      ]
    }


### Evaluation

The stubs are evaluated in the order they are defined in. The first stub that
has a matching predicate will generate the response.


## Web API (REST)

The normal way to interact with Candouble is via its web API. It allows posting
and retrieving of stubs. (An alternative is to specify files containing imposter
definitions when starting Candouble.)


### Adding and updating imposters

Imposter definitions can be posted to the `/imposters` endpoint, e.g.

    curl -i -X POST -H 'Content-Type: application/json' http://localhost:8080/imposters ↩
    --data '{ "id": 0, "stubs": [ { "predicates": [{ "eq": { "id": "0x01" } }], "responses": [{ "id": "0x02", "data": [ "0x01" ] }] } ] }'

Unless something goes wrong, the API should respond with status code `201
CREATED`. If an imposter with the given id exists already, that imposter will be
replaced. In that case the response is  `200 OK`.


### Retrieving a specific imposter

Imposters can be retrieved by their CAN port id, e.g.

    curl -i http://localhost:8080/imposters/0

To allow following REST principles strictly, knowledge of this URL format is
not actually necessary. The response to POSTing an imposter includes a
`Location` header that contains the URL for the imposter. That said, you can
safely use URLs with the format documented here, though.

When the `recordMessages` field is set to `true`, the imposter records all
incoming messages and these are then included in the response, e.g.

	{
	  "id": 0,
	  "recordMessages": true,
	  "stubs": [
	    {
	      "predicates": [
	        { "eq": { "id": "0x1" } }
	      ],
	      "responses": [
	        { "id": "0x201", "data": [ "0x01" ], "_behaviors": null
	        }
	      ]
	    }
	  ],
	  "messages": [
	    {
	      "id": 1,
	      "type": 1,
	      "length": 2,
	      "data": [ 202, 254, 0, 0, 0, 0, 0, 0 ]
	    }
	  ]
	}

Note that the `data` field of recorded messages always contains eight values.
Also note that recording of messages is turned off by default, because this
effectively represents a memory leak for long-running imposters.


### Retrieving all imposters

A list of all imposters can also be retrieved, e.g.

    curl -i http://localhost:8080/imposters

The list of imposters is wrapped in a top-level object, e.g.

    {
      "imposters": [
        {
          "id": 0,
          "stubs": [
    ...
          ]
        }
      ]
    }    


### Removing an imposter

An imposter can be removed using the `DELETE` HTTP verb, e.g.

    curl -i -X DELETE http://localhost:8080/imposters/0

Note that the API does not return the deleted imposter and therefore responds
with status code `204 NO CONTENT`.


## CAN hardware adaptors

If you're on a Mac and have the PCAN adaptor attached, you should run the
application with the `pcan` feature. For it to find the native library you have
to set the dynamic library loading path:


    export LD_LIBRARY_PATH=./lib/PCBUSB
    cargo run --features pcan tests/it_imposter.json

If you're not on a Mac then you can run the unit tests, but there are no
adaptors yet for CAN hardware.



