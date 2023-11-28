# Impl Display For Group

| Nombre alumno          | Padron | Mail                      | Github                                     |              
|------------------------|--------|---------------------------|--------------------------------------------|
| Agustin Ariel Andrade  | 104046 | aandrade@fi.uba.ar        | [GitHub](https://github.com/AgussAndrade)  |
| Carolina Di Matteo     | 103963 | cdimatteo@fi.uba.ar       | [GitHub](https://github.com/gcc-cdimatteo) |
| Tomas Apaldetti        | 105157 | apaldettitomasl@gmail.com | [GitHub](https://github.com/tomas-L-Apal)  |
| Valentina Laura Correa | 104415 | vcorrea@fi.uba.ar         | [GitHub](https://github.com/valencorrea)   |


### Introducción
La presente entrega contiene las funcionalidades pedidas para el trabajo practico nº2 de la materia Taller de Programacion I - curso Deymonnaz.

### Objetivo
El objetivo del presente Trabajo Práctico consiste en el desarrollo de un servidor y un cliente de chat siguiendo los lineamientos del protocolo [IRC](https://es.wikipedia.org/wiki/Internet_Relay_Chat), implementandolo en [Rust](https://doc.rust-lang.org/rust-by-example/index.html) y siguiendo los conceptos trabajados en clase.
La definición base de como funciona esta dada en documentos [RFCs](https://www.rfc-editor.org/rfc/rfc1459).
### Ejecución
- cargo run server <puerto>
- cargo run client <ip cliente> <puerto server>
- cargo run server-connect <puerto nuevo server> <ip server a conectar> <puerto server a conectar> <contraseña>

Otros comandos de interes:
- *cargo test*
- *cargo fmt*
- *cargo clippy*
- *cargo doc --open*

### Otros Links
- [Enunciado](https://taller-1-fiuba-rust.github.io/proyecto/22C2/proyecto.html)
- [Diagramas](https://app.diagrams.net/#G1LDzCBdfU6UKWk-cUuacOXrRNGc4KU1cd)
- [Trello](https://trello.com/b/GN3v06p7/irc)
