# Server
Usage: **modu server [port: default 2424]**

This will start the built-in modu interpreter server, which can be used to run code on platforms not supported by modu, like the web! \
An example of this is the [Modu Web IDE](../ide).

The server has the followings endpoints:
```
GET  /     - Can be used to check if server on
POST /eval - Used to run code
```

In order to run some code, make a post requets to /eval with the code as raw text in the body, like:
```bash
curl --location 'http://localhost:2424/eval' \
     --header 'Content-Type: text/plain' \
     --data 'let a = 1;
     print(a);

     print("YOOOOO");

     if a == 1 {
         print("LES GOOOO");
     }

     print(a);'
```

This will return a response in plaintext like

```
1
YOOOOO
LES GOOOO
1

```

## What wont work?
**input()** will not work, as when running code its sent to the server, executed there, and sent back. And we have currently not added any way to make input() work, which would be extremely hard. \
**exit()** has been disabled so people dont try to crash the server.



The following (built-in) packages has been disabled on the server:
- OS
- File
- FFI