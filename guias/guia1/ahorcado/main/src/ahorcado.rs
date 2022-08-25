use std::io;

pub fn jugar_ahorcado(palabra: &String){
    let cantidad_intentos = 6;
    let mut intentos_fallidos = 0;
    let letras_totales = palabra.len();
    let mut letras_adivinadas = 0;

    println!("Bienvenido al ahorcado de FIUBA!");

    while ((cantidad_intentos - intentos_fallidos) > 0) ||
            (letras_adivinadas != letras_totales) {

        mostrar_reporte_actual();
        let mut nueva_letra = pedir_nueva_letra();
        let adivino = adivinar_letra(nueva_letra);

        if adivino {
            letras_adivinadas = letras_adivinadas +1;
        }
        else{
            intentos_fallidos = intentos_fallidos +1;

        }

    }   
    mostrar_reporte_final(letras_totales, letras_adivinadas); 
}

pub fn adivinar_letra(nueva_letra: String) -> bool {
    true
}

pub fn mostrar_reporte_final(letras_totales: usize, letras_adivinadas: usize){
    if letras_totales == letras_adivinadas {
        println!("Felicitaciones, ganaste!!!");
    }
    else {
        println!("Game over :( Suerte la prox");
    }

}

pub fn mostrar_reporte_actual(){
    //mostrar_palabra_actual();
    //mostrar_letras_adivinadas();
    //mostrar_cantidad_intentos();
}

pub fn pedir_nueva_letra() -> String{
    let stdin = io::stdin();
    let mut letra = String::new();
    println!("Ingresa una letra:");
    stdin.read_line(&mut letra).unwrap();
    letra
}

/*
La palabra hasta el momento es: _ _ _ _ _ _
Adivinaste las siguientes letras:
Te quedan 5 intentos.
Ingresa una letra: r

La palabra hasta el momento es: _ _ _ _ _ r
Adivinaste las siguientes letras: r
Te quedan 5 intentos.
Ingresa una letra: c
*/