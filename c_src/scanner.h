#pragma once

#include "sdk_api.h"
#include "log.h"

#include <vector>

/*
		#########################
		#						#
		#	 SAMPLES SCANNER	#
		#						#
		#########################


	Task: Class for management with SensorScanner 
	(searching devices and connection to one of them).

*/

class SampleScanner
{
public:
	//Constructor of class
	SampleScanner();

	//Destructor of class
	~SampleScanner();

	//Create object SensorScanner for seaching devicesand get list of found ones.
	bool CreateScanner(SensorFamily* ListFilters, int32_t CountFilters);

	//Begin searching devices.
	bool StartScanner();

	//Finish searching devices.
	bool StopScanner();

	//Get list of all found devices by scanning.
	std::vector<SensorInfo> GetDevices();

	bool AddSensorsCallbackScanner();
	bool RemoveSensorsCallbackScanner();

	//Create object of device for its controlling.
	Sensor* CreateSensor(SensorInfo device);

private:
	//Object Scanner (need for scanning devices).
	SensorScanner* _scanner;

	//SensorListener is necessary for get target callback and remove it.
	SensorsListenerHandle _handle = 0;
};