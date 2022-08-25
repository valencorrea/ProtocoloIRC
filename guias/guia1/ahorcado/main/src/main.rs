mod ahorcado;

use crate::ahorcado::jugar_ahorcado;

fn main() {
    
    //let palabra = String::from(palabras); //como leo del archivo?
    let palabra = String::from("casa");
    jugar_ahorcado(&palabra);
    
}