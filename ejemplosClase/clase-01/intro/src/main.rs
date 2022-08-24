mod correlativas;
mod sort;

use crate::sort::saludar_nombre;

fn main() {
    //saludar();
    let nombre = String::from("Pablo");

    saludar_nombre(&nombre);
}
