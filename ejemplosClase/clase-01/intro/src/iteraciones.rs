use crate::sort::saludar_nombre;

#[test]
pub fn iterar() {
    let lista_nombres = vec![
        String::from("Iñaki"),
        String::from("Matías"),
        String::from("Uriel"),
        String::from("Juan"),
    ];

    for i in 0..lista_nombres.len() {
        saludar_nombre(&lista_nombres[i]);
    }

    // Ojo, usamos el & para no consumir la lista, esto es tema de clase 2
    // (intrinsecamente hay un into_iter())
    // Iterar usando for __ in ___
    for nombre in &lista_nombres {
        saludar_nombre(nombre);
    }
    // Iterar usando iter() foreach( closure )
    lista_nombres
        .iter()
        .for_each(|nombre| saludar_nombre(nombre));
    // Equivalente a la de arriba
    lista_nombres.iter().for_each(saludar_nombre);
}
