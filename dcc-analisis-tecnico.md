# **DCC**
1. Entro a IRC como caro
2. Entro a IRC como valen

## **DCC CHAT**
1. caro envía *NOTICE valen :0x1 CTCP valen DCC CHAT CHAT <ip_caro> <puerto_caro> 0x1*
2. caro abre el socket pasivo
3. valen recibe el mensaje
	### **on accept**
    1. valen se conecta al puerto recibido de la ip recibida
    2. caro cierra el socket pasivo
    3. caro abre el nuevo socket
    4. comienza la transferencia de datos
	### **on deny**
    1. valen envía *NOTICE caro :0x1 DCC DENY 0x1*
    2. caro recibe el mensaje
    3. caro cierra el socket pasivo

## **DCC SEND**
1. caro envía *NOTICE valen :0x1 CTCP valen DCC SEND SEND <file_name> <ip_caro> <puerto_caro> <file_size> 0x1*
2. caro abre el socket pasivo
3. valen recibe el mensaje
	### **on accept**
    1. valen envía *NOTICE caro :0x1 CTCP caro DCC RESUME <file_name> <puerto_caro> <0> 0x1*
    2. caro recibe el mensaje
    3. caro cierra el socket pasivo
    4. caro abre el nuevo socket
    5. comienza la transferencia de datos hasta finalizar el envío del archivo o hasta que se produzca un error y se frene la transferencia
	### **on deny**
    1. valen envía *NOTICE caro :0x1 DCC DENY 0x1*
    2. caro cierra el socket pasivo

## **DCC RESUME**
### **when acceptor —> works like accepting the file**
1. valen envía *NOTICE caro :0x1 CTCP caro DCC RESUME <file_name> <puerto_caro> <position> 0x1*
2. caro recibe el mensaje
3. caro cierra el socket pasivo
4. caro abre el nuevo socket
5. comienza la transferencia de datos hasta finalizar el envío del archivo o hasta que se produzca un error y se frene la transferencia

## **DCC CLOSE**
### **when initiator**
1. caro envía *DCC CLOSE* (gtk envía 0x1 CTPC DCC CLOSE 0x1)
2. caro cierra el socket
3. caro cierra el thread de escritura y lectura
4. valen recibe el mensaje
5. valen se desconecta de caro
6. valen cierra el thread de escritura y lectura
### **when acceptor**
1. valen envía *DCC CLOSE* (gtk envía 0x1 CTPC DCC CLOSE 0x1)
2. valen se desconecta de caro
3. valen cierra el thread de escritura y lectura
4. caro recibe el mensaje
5. caro cierra el socket
6. caro cierra el thread de escritura y lectura

## ACLARACIONES
* DCC CHAT on accept —> punto 9:
    * se lee y escriben strings
    * se interceptan todos los mensajes en busca de un 0x1 DCC CLOSE 0x1
* DCC SEND on accept —> punto 7:
    * en caso que el archivo contenga el string “0x1 DCC CLOSE 0x1” que coincidiría con el mensaje de terminación de transferencia de datos que definimos para el protocolo, deberá reanudarse la transferencia de datos en una posición posterior al archivo en cuestión. De otra forma continuará infinitamente cerrándose el canal de conexión y no podrá finalizarse el envío del archivo

