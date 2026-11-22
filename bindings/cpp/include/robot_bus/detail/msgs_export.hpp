#pragma once

#if defined(_WIN32)
#if defined(ROBOT_BUS_MSGS_BUILDING_DLL)
#define ROBOT_BUS_MSGS_API __declspec(dllexport)
#else
#define ROBOT_BUS_MSGS_API __declspec(dllimport)
#endif
#else
#define ROBOT_BUS_MSGS_API
#endif
