# SQML

  ______  __       __  ______  __       
 /      \|  \     /  \/      \|  \      
|  ▓▓▓▓▓▓\ ▓▓\   /  ▓▓  ▓▓▓▓▓▓\ ▓▓      
| ▓▓___\▓▓ ▓▓▓\ /  ▓▓▓ ▓▓  | ▓▓ ▓▓      
 \▓▓    \| ▓▓▓▓\  ▓▓▓▓ ▓▓  | ▓▓ ▓▓      
 _\▓▓▓▓▓▓\ ▓▓\▓▓ ▓▓ ▓▓ ▓▓ _| ▓▓ ▓▓      
|  \__| ▓▓ ▓▓ \▓▓▓| ▓▓ ▓▓/ \ ▓▓ ▓▓_____ 
 \▓▓    ▓▓ ▓▓  \▓ | ▓▓\▓▓ ▓▓ ▓▓ ▓▓     \
  \▓▓▓▓▓▓ \▓▓      \▓▓ \▓▓▓▓▓▓\\▓▓▓▓▓▓▓▓
                           \▓▓▓         
                                              


## what

smol in-memory mq in rust
supports add, get/consume, delete/ack, retry, purge, and peek operations.
includes example producer and consumer web UIs.

### message lifecycle

messages in SMQL move through the following distinct states:
1. **ready**: available for consumers to retrieve
1. **processing**: locked by consumer, invisible to others (peek as workaround for visibility, only for demoing)

### message structure

```
{
  "id": "uuid",
  "body": "string",
  "state": "Ready" | "Processing",
  "lock_until": null,
  "retry_count": 0
}
```

### message processing pattern

1. consumer retrieves message via /get
2. message locks automatically (processing state)
3. consumer processes the message
4. on success: delete message via /delete
5. on failure: return to queue via /retry

### shortcomings

- no persistence - all messages lost on server restart
- no TTL - Messages remain until explicitly deleted
- no dead letter queue - retry count increments but messages retry indefinitely

## operations || api reference

### add
**POST /add**
```json
{"body": "text"}
```
returns:
```json
{
  "id": "uuid",
  "body": "text",
  "state": "ready",
  "retry_count": 0
}
```

### get  
**POST /get**
```json
{"count": 5}
```
returns messages and marks them as:
- `processing`
- invisible until deleted or retried

```json
[
  {
    "id": "uuid",
    "body": "text",
    "state": "processing",
    "retry_count": 0
  }
]
```

### delete  
**POST /delete**
```json
{"ids": ["uuid1", "uuid2"]}
```
permanently removes messages.

### retry  
**POST /retry**
```json
{"ids": ["uuid1", "uuid2"]}
```
moves messages back to `ready` and increments `retry_count`.

### purge  
**POST /purge**
```json
{}
```
clears all messages.

### peek  
**POST /peek**
```json
{"count": 5}
```
returns messages **without changing** state or visibility.

```json
[
  {
    "id": "uuid",
    "body": "text",
    "state": "ready",
    "retry_count": 0
  }
]
```


## basic workflow

```bash
# add message
curl -X POST localhost:1337/add \
  -H "Content-Type: application/json" \
  -d '{"body":"Process this task"}'

# get and process
curl -X POST localhost:1337/get \
  -H "Content-Type: application/json" \
  -d '{"count":1}'

# peek the queue
curl -X POST localhost:1337/peek \
  -H "Content-Type: application/json" \
  -d '{"count":1}'

# success - delete message
curl -X POST localhost:1337/delete \
  -H "Content-Type: application/json" \
  -d '{"ids":["returned-uuid-here"]}'

# failure - retry message
curl -X POST localhost:1337/retry \
  -H "Content-Type: application/json" \
  -d '{"ids":["returned-uuid-here"]}'
```

## run

to start smql
```bash
cargo run
```

to view the webserver demo,
```bash
cd ./static
npx serve
```

producer and consumer files at `./producer.html` and `./consumer.html`