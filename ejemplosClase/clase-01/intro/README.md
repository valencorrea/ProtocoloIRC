# Práctica 01

## Descargar compilador

https://www.rust-lang.org/learn/get-started -> instalación default 

Ejecutar `source $HOME/.cargo/env` para poder empezar

## Cargo

**Cargo** es el manejador de paquetes de Rust. Posee varios comandos para compilar y ejecutar nuestros proyectos, además de gestionar paquetes de software externo.

Algunos comandos útiles:

`cargo new <nombre>`: crea una carpeta con el nombre `<nombre>`. Dentro de la misma se encuentra un "hola mundo" y un repositorio git.

`cargo build`: compila nuestro proyecto

`cargo run`: ejecuta el proyecto, lo compila si es necesario

`cargo test`: ejecuta los tests presentes en el proyecto.

`cargo test -- --nocapture `: ejecuta tests, pero mostrando las escrituras a stdout (es decir, muestra los `println`)

`cargo doc`: compila la documentación del proyecto. `cargo doc --open` la abre en un navegador

Información relevante al proyecto puede ser configurada en el archivo `cargo.toml`

> Dato curioso: para el proceso de linkeo, cargo necesita un linker de C, así que recordar instalarlo con `sudo apt install build-essential`

## Rustup

`rustup` es nuestro gestor de *toolchains*

Un *toolchain* es un conjunto de herramientas utilizados en el proceso de compilación.

Cuando instalamos Rust, instalamos el toolchain "estable". Sin embargo, algunos *features* del lenguaje están disponible sólamente en el toolchain inestable o "nightly".

Algunos comando útiles son:

* `rustup update` actualizar toolchain y componentes
* `rustup default` -> `rustup default niglty|stable|<nombre>` elegir toolchain
* `rustup component list`: componentes que pertenecen al toolchain

### Precauciones

*Racer* es una herramienta de autocompletado de código disponible en los repositorios de cargo. El mismo se instala utilizando el toolchain *nightly* debido a que usa muchos features inestable.

Al instalar racer, necesario para algunos IDEs, habían dependencias nightly que estaban rotas: *rustc-ap-rustc_span* no compilaba porque dependía de código inestable, entre ellos un feature que habilitaba la funcion `expect_none()` sobre un Option.

¿Qué pasó en una semana de marzo 2021? Estas funciones fueron retiradas del compilador: https://github.com/rust-lang/rust/pull/83349

Solución frente a este tipo de problemas: `rustup` nos permite instalar versiones específicas de nightly. En este caso, se pudo instalar una versión del compilador previo a que retiren esas funciones inestables.

`rustup toolchain install nightly-2021-03-13` (nightly 1.52)


## Ejemplos de sintaxis

Compilar utilizando `cargo build`

Generar y abrir la documentación utilizando `cargo doc --open`
