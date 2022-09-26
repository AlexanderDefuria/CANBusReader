#include <Arduino.h>
#include <SPI.h> //Library for using SPI Communication
#include <mcp2515.h> //Library for using CAN Communication

struct can_frame canMsg;
MCP2515 mcp2515(10); // SPI CS Pin 10
struct can_frame canMsg1;


void setup() {
    SPI.begin();   //Begins SPI communication
    Serial.begin(9600); //Begins Serial Communication at 9600 baud rate
    mcp2515.reset();
    mcp2515.setBitrate(CAN_1000KBPS,MCP_20MHZ); //Sets CAN at speed 500KBPS and Clock 8MHz
    mcp2515.setNormalMode();  //Sets CAN at normal mode
    Serial.print("HIT");

    canMsg1.can_id  = 0x02;
    canMsg1.can_dlc = 8;
    canMsg1.data[0] = 0x8E;
    canMsg1.data[1] = 0x87;
    canMsg1.data[2] = 0x32;
    canMsg1.data[3] = 0xFA;
    canMsg1.data[4] = 0x26;
    canMsg1.data[5] = 0x8E;
    canMsg1.data[6] = 0xBE;
    canMsg1.data[7] = 0x86;
}

void loop(){
//    if ((mcp2515.readMessage(&canMsg) == MCP2515::ERROR_OK)){
//        Serial.println(canMsg.can_id, 2);
//        for (unsigned char i : canMsg.data)
//            Serial.println(i);
//        Serial.println("");
//    }

    mcp2515.sendMessage(&canMsg1);


}
