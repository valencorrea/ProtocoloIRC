use std::io;

pub fn jugar_ahorcado(palabra: &String){
    let mut intentos_restantes = 6;
    let mut letras_por_adivinar = palabra.len();

    println!("Bienvenido al ahorcado de FIUBA!");

    while (intentos_restantes > 0) && (letras_por_adivinar > 0) {
        mostrar_reporte_actual();

        if !adivinar_letra(pedir_nueva_letra()) {
            intentos_restantes = intentos_restantes - 1;
        }
        else{
            letras_por_adivinar = letras_por_adivinar - 1;
        }
    }   
    mostrar_reporte_final(letras_por_adivinar); 
}

pub fn adivinar_letra(nueva_letra: String) -> bool {
    true
}

pub fn mostrar_reporte_final(letras_por_adivinar: usize){
    if letras_por_adivinar == 0 {
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