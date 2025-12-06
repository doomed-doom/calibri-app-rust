#include "scanner.h"

#include "sensorsCallback.h"

SampleScanner::SampleScanner()
{
	//Constructor of class
	//This functions will be called when you create class.
	//
	//For examples,
	//SamplesCanner* scan = new SamplesScanner();
	//
	//In this moment 'scan' - object of class 'SamplesScanner' will call costructor.
}

SampleScanner::~SampleScanner()
{
	//Destructor of class
	//This functions will be called when you delete class.
	//
	//For examples,
	//delete(scan);
	//
	//In this moment 'scan' - object of class 'SamplesScanner' will call destructor.

	//ALWAYS FREE MEMORY OF OBJECTS!

	try
	{
		//Check if scanner is empty of not.
		if (_scanner != nullptr)
		{
			//For safety clear Scanner we should use function 'freeScanner'.
			//Function accepts target scanner.
			freeScanner(_scanner);
		}

		//In this code we will delete all handle of callback 
		//in case the user forgot to delete them.
		//Always delete handle and pointers in class, which 
		//will not be used in future or in situation, when class call destructor.

		//Check if Handle of found devices is empty or not
		if (_handle != 0)
		{
			//Remove handle with function.
			removeSensorsCallbackScanner(_handle);
		}
	}
	catch (std::exception error)
	{
		//Print Error Message on Console.
		EConsole::PrintScreen("[[ERROR : ", error.what(), " ]]");
	}
	catch (...)
	{
		//Print Error Message on Console (unknown error).
		EConsole::PrintScreen("[[ERROR : Invalid error! ]]");
	}
}

/*
		#####################
		#					#
		#	CREATE SCANNER	#
		#					#
		#####################


	Task: Create object SensorScanner for
	seaching devices and get list of
	found ones.

*/

bool SampleScanner::CreateScanner(SensorFamily* ListFilters, int32_t CountFilters)
{
	try
	{
		//ListFilters - array of devices, which we want to find.
		//CountFilters - size of array 'ListFilters'.

		if (ListFilters == nullptr)
		{
			throw std::invalid_argument("Null list of filters!");
		}

		//Create boolean for get information of successful or failed operation (creating scanner).
		OpStatus outStatus;

		//Target _scanner - SensorScanner. SensorScanner - object, which need for searching devices.
		//Create scanner for searching devices. 
		//Function 'createScanner' accepts SensorFamily (List of need devices), size of list. 
		//Function return object Scanner & boolean result ('outStatus').
		_scanner = createScanner(ListFilters, CountFilters, &outStatus);

		//Check is successful creating scanner OR creating scanner return null.
		if (!outStatus.Success && _scanner == nullptr)
		{
			throw std::invalid_argument(outStatus.ErrorMsg);
		}

		//If you don't use scanner sooner or your programm stops,
		//you should delete all targets using special
		//function 'freeScanner'.

		//freeScanner(_scanner);

		//Return true value means our algorythm is working correct.
		return true;
	}
	catch (std::exception error)
	{
		//Print Error Message on Console.
		EConsole::PrintScreen("[[ERROR : ", error.what(), " ]]");
		//Return false value means our algorythm is working incorrect.
		return false;
	}
	catch (...)
	{
		//Print Error Message on Console (unknown error).
		EConsole::PrintScreen("[[ERROR : Invalid error! ]]");
		//Return false value means our algorythm is working incorrect.
		return false;
	}
}

/*
		#####################
		#					#
		#	START SCANNER	#
		#					#
		#####################


	Task: Begin searching devices.

*/

bool SampleScanner::StartScanner()
{
	try
	{
		//Check if our scanner is empty or not.
		if (_scanner == nullptr)
		{
			throw std::invalid_argument("Null scanner!");
		}

		//Create boolean for get information of successful or failed operation (searching devices).
		OpStatus outStatus;

		//Start searching devices. 
		//Function 'startScanner' accepts Scanner (which we get from function 'createScanner').
		//Function return boolean result and OpStatus for getting error (if it will be).
		bool result = startScanner(_scanner, &outStatus);

		//Function 'startScanner' print in Console all log searching devices.
		//For example:
		// [2022-09-09 12:44:26.095] [sdk_log] [trace] [BLE_ADV][][7A:A3:A9:4A:A8:E0] - no device
		//
		// OR
		// 
		// [2022-09-09 12:45:41.170] [sdk_log] [trace] [BLE_ADV][Callibri_Red][D8:46:9A:3F:42:57] - our Callibri device

		//Check is successful starting searching devices
		if (!result)
		{
			throw std::invalid_argument(outStatus.ErrorMsg);
		}

		//After current time you should stop scanner. This example was shown in function 'StopScanner'.

		//Return true value means our algorythm is working correct.
		return true;
	}
	catch (std::exception error)
	{
		//Print Error Message on Console.
		EConsole::PrintScreen("[[ERROR : ", error.what(), " ]]");
		//Return false value means our algorythm is working incorrect.
		return false;
	}
	catch (...)
	{
		//Print Error Message on Console (unknown error).
		EConsole::PrintScreen("[[ERROR : Invalid error! ]]");
		//Return false value means our algorythm is working incorrect.
		return false;
	}
}

/*

		#####################
		#					#
		#	STOP SCANNER	#
		#					#
		#####################


	Task: Finish searching devices.

*/

bool SampleScanner::StopScanner()
{
	try
	{
		//Check if our scanner is empty or not.
		if (_scanner == nullptr)
		{
			throw std::invalid_argument("Null scanner!");
		}

		//Create boolean for get information of successful or failed operation (stop searching devices).
		OpStatus outStatus;

		//Finish searching devices.
		//Function 'stopScanner' accepts Scanner (which we get from function 'createScanner').
		//Function return boolean result and OpStatus
		bool result = stopScanner(_scanner, &outStatus);

		//Check is successful stop searching devices
		if (!result)
		{
			throw std::invalid_argument(outStatus.ErrorMsg);
		}

		//Return true value means our algorythm is working correct.
		return true;
	}
	catch (std::exception error)
	{
		//Print Error Message on Console.
		EConsole::PrintScreen("[[ERROR : ", error.what(), " ]]");
		//Return false value means our algorythm is working incorrect.
		return false;
	}
	catch (...)
	{
		//Print Error Message on Console (unknown error).
		EConsole::PrintScreen("[[ERROR : Invalid error! ]]");
		//Return false value means our algorythm is working incorrect.
		return false;
	}
}

/*
		#########################
		#						#
		#	GET FOUND DEVICES	#
		#						#
		#########################


	Task: Get list of all found devices
	by scanning.

*/

std::vector<SensorInfo> SampleScanner::GetDevices()
{
	try
	{
		//Check if our scanner is empty or not.
		if (_scanner == nullptr)
		{
			throw std::invalid_argument("Null scanner!");
		}

		//countDevices - count of all found devices.
		int32_t countDevices = 1;
		//devices - array of all found devices. Create target SensorInfo and set size.
		SensorInfo* devices = new SensorInfo[countDevices];

		//Create boolean for get information of successful or failed operation (get list of found devices).
		OpStatus outStatus;

		//Get All Found Devices from scanning.
		//Function 'sensorsScanner' accepts Scanner.
		//Function returns boolean result, array SenserInfo (information of devices), count of devices and OpStatus.
		bool result = sensorsScanner(_scanner, devices, &countDevices, &outStatus);

		//Check is successful get found devices
		if (!result)
		{
			throw std::invalid_argument(outStatus.ErrorMsg);
		}

		//listDevices - list of all found devices. We use vector for comfortables using lists.
		std::vector<SensorInfo> listDevices;

		//Fill our vector with elements from array 'devices'.
		for (int i = 0; i < countDevices; i++)
		{
			listDevices.push_back(devices[i]);
		}

		//Delete array 'devices'. We won't use with array, so we need to delete it.
		delete[] devices;

		//Return vector of devices.
		return listDevices;
	}
	catch (std::exception error)
	{
		//Print Error Message on Console.
		EConsole::PrintScreen("[[ERROR : ", error.what(), " ]]");
		//Return empty vector value means our algorythm is working incorrect.
		return {};
	}
	catch (...)
	{
		//Print Error Message on Console (unknown error).
		EConsole::PrintScreen("[[ERROR : Invalid error! ]]");
		//Return empty vector value means our algorythm is working incorrect.
		return {};
	}
}

/*
		#############################
		#							#
		#	ADD SENSORS CALLBACK	#
		#							#
		#############################


	Task: Add callback to get information of
	new found devices.
*/

bool SampleScanner::AddSensorsCallbackScanner()
{
	try
	{
		//Check if our scanner is empty or not.
		if (_scanner == nullptr)
		{
			throw std::invalid_argument("Null scanner!");
		}

		//Create boolean for get information of successful or failed operation
		OpStatus outStatus;

		//To add callback for searching new devices we can use function 'addSensorsCallbackScanner'.
		//Functions accepts object of scanner, function, which will be called for getting information.
		//Functions return boolean result, handle of callback and OpStatus.
		bool result = addSensorsCallbackScanner(_scanner, sensorsCallback, &_handle, nullptr, &outStatus);

		//Check result of adding callback, is successful or not. 
		if (!result)
		{
			throw std::invalid_argument(outStatus.ErrorMsg);
		}

		//Return true result of adding callback.
		return true;
	}
	catch (std::exception error)
	{
		//Print Error Message on Console.
		EConsole::PrintScreen("[[ERROR : ", error.what(), " ]]");
		//Return false value means our algorythm is working incorrect.
		return false;
	}
	catch (...)
	{
		//Print Error Message on Console (unknown error).
		EConsole::PrintScreen("[[ERROR : Invalid error! ]]");
		//Return false value means our algorythm is working incorrect.
		return false;
	}
}

/*
		#############################
		#							#
		#	REMOVE SENSORS CALLBACK	#
		#							#
		#############################


	Task: remove callback for searching new
	device.

*/

bool SampleScanner::RemoveSensorsCallbackScanner()
{
	try
	{
		//Check if our scanner is empty or not.
		if (_scanner == nullptr)
		{
			throw std::invalid_argument("Null scanner!");
		}

		//Check if our handle is empty or not.
		if (_handle == 0)
		{
			throw std::invalid_argument("Null Sensors callback!");
		}

		//Remove handle with function.
		removeSensorsCallbackScanner(_handle);

		//Return true value of successfule removing.
		return true;
	}
	catch (std::exception error)
	{
		//Print Error Message on Console.
		EConsole::PrintScreen("[[ERROR : ", error.what(), " ]]");
		//Return false value means our algorythm is working incorrect.
		return false;
	}
	catch (...)
	{
		//Print Error Message on Console (unknown error).
		EConsole::PrintScreen("[[ERROR : Invalid error! ]]");
		//Return false value means our algorythm is working incorrect.
		return false;
	}
}

/*
		#####################
		#					#
		#	CREATE SENSOR   #
		#					#
		#####################


	Task: After scanning and getting
	list of found device you need to
	choose one of device to control
	it and change.

*/

Sensor* SampleScanner::CreateSensor(SensorInfo device)
{
	try
	{
		//Check if our scanner is empty or not.
		if (_scanner == nullptr)
		{
			throw std::invalid_argument("Null value SensorScanner!");
		}

		//Create boolean for get information of successful or failed operation
		OpStatus outStatus;

		//To create object of device we need to use function 'createSensor'.
		//Function accepts object of scanner, sensor info of need device.
		//Function return boolean result of work and object of device (Sensor).
		Sensor* result = createSensor(_scanner, device, &outStatus);

		//Check result is success or not & object of device is empty or not
		if (!outStatus.Success && result == nullptr)
		{
			throw std::invalid_argument(outStatus.ErrorMsg);
		}

		//Return object of device
		return result;
	}
	catch (std::exception error)
	{
		//Print Error Message on Console.
		EConsole::PrintScreen("[[ERROR : ", error.what(), " ]]");
		//Return false value means our algorithm is working incorrect.
		return nullptr;
	}
	catch (...)
	{
		//Print Error Message on Console (unknown error).
		EConsole::PrintScreen("[[ERROR : Invalid error! ]]");
		//Return false value means our algorithm is working incorrect.
		return nullptr;
	}
}