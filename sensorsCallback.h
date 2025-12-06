#pragma once

#include "sdk_api.h"
#include "log.h"

void sensorsCallback(SensorScanner* scanner, SensorInfo* devices, int32_t countDevices, void* userData)
{
	for (int i = 0; i < countDevices; i++)
	{
		EConsole::PrintLog("[SENSOR] [NAME] : ", devices[i].Name, " [SERIAL NUMBERS] :", devices[i].SerialNumber, " [ADDRESS] :", devices[i].Address);
	}
}