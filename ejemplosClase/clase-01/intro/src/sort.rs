use std::io;

/// Pide lee una linea por stdin e imprime el nombre stdout
pub fn saludar() {
    let stdin = io::stdin();
    let mut nombre = String::new();
    println!("Ingrese su nombre");
    stdin.read_line(&mut nombre).unwrap();
    println!("Nombre: {:?}", nombre);
}

/// Imprime por stdout un saludo
pub fn saludar_nombre(nombre: &String) {
    println!("Hola {:?}", nombre);
}

/// Implementación sencilla de insertion sort
pub fn sort(nums: &Vec<u32>) -> Vec<u32> {
    let mut result = nums.clone();
    for i in 0..result.len() {
        let mut min = i;
        for j in i..result.len() {
            if result[j] < result[min] {
                min = j;
            }
        }
        result.swap(i, min);
    }
    result
}
