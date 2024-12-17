# TUIFS
rust tui fileserver 

## Todo
client 
- make it pretty - backgrounds 
- add error handling for incorrectly inputted values 
- add error handling for incorrect server responses 
- add upload file functionality
    - add upload folder functionality (hyper buffered response)

server 
- add recieve file/folder functionality 
- add send file functionality 


## notes / ideas
    jotting stuff / general ideas down post-ratatui tutorial

Client - App struct
- vector of strings representing files/folders on the server - stateful list 
    - users can pull files from the server using this
    - popup for install dir 
- hints for buttons to sync with the server 
- add/upload file/folder to server by directory
    - eventual nice ui for this
    - if stringpath is dir - upload dir
    - if is file - upload file 
    - can replace folders on server if name is match

Server 
endpoints:
- upload files/folder
    - how does uploading work?
- list files/folders 
- download files/folder 
    - how to send from client?

Shared:
- ...



