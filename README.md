![banner](https://github.com/Raregendary/SimpleBeatmapAnalyzer/assets/71941668/2ef72068-7cc7-4c05-bb12-a5ee62e84d08)


# What is Simple Beatmap Analyzer?
###### This is my first rust program
Simple Beatmap Analyzer is a console application that analyzes ".osu" files for Standard game mode only. After it finishes it outputs the results in a "CSV" *(comma,separated,values)* file in the same directory as the .exe
# Table of Contents
1. [How does it work](#how-does-it-work)
1. [How to install](#how-to-install)
1. [How to use it](#how-to-use-it)
1. [How to load results.csv in Excel](#how-to-load-resultscsv-in-excel)
1. [How to analyze the data in Excel](#how-to-analyze-the-data-in-excel)
1. [To Do](#to-do)
1. [F.A.Q](#faq)


# How does it work
It assumes every map is in maped **1/4** and calculates streams/jumps and other patterns aswel as some ratios based on them *(Jumps are asumed every second beat and i will be refering to it as 1/2)*.
The values for the following **1/4** and **1/2** sections are in % notes of the map, not in time.
* For **1/4** patterns we assume the circles are overlapping or up to 16 pixels edge to edge.
* For **1/2** patterns we asume the edge to edge space is higher than 110 pixels
1. **1\4** patterns:
	* *Dobules/Triples/Quads/Bursts/Streams/DeathStreams*
	* Bursts:      **3-12** *includes Triples and Quads*
	* Streams:	**13-32**
	* DeathStreams: **33+**
1. **1\2** patterns:
	* *ShortJumps/MidJumps/LongJumps*
	* ShortJumps:  **3-12**
	* MidJumps: **13-32**
	* LongJumps: **33+**
1. **SI** -> Stream index *(how streamy the map is minus jumps)*
1. **JI** -> Jump index *(how jumpy the map is minus streams)*
1. **FCDBI** -> Finger Control Double Bursts Index *(experimental)*
1. Additionals stats like 99% acc PP for NM/DT/HR aswel as Stars for said mods and others... *wont list em all*
# How to install
1. Click on releases and download the latest version.
1. Unzip into a folder of your choice.

# How to use it
1. Open the **SimpleBeatmapAnalyzer.exe**
1. Paste the Osu Songs path *(example: D:\osu\Songs)* and click Enter
1. Wait for it to finish and save the data in **results.csv**
1. Open the results.csv in Excel or any other CSV reader/analyzer
1. After you find the map you like simply copy the **MapID** and paste it in the Osu search ingame

# How to load results.csv in Excel
#### Method 1
1. Open a new excel document.
1. Chose the menu **Data** and from there click **From Text/CSV**
1. Select the **results.csv** and click **Open**
1. After then on the first drop down menu select **65001: Unicode (UTF-8)** if its not auto selected
1. Make sure the delimiter is **Comma** and press **Load**
---
#### Method 2
1. If in windows your default delimiter is "," and not "." open the csv with Excel
1. Chose the menu **Insert** and from there click **Table**
1. It should auto detect the scope of your table, if not manualy select it
1. Enable **My table has headers** checkbox and press **Ok**

# How to analyze the data in Excel
#### Example 1: Most Streamy 7\*  Beatmaps from 210 to 220 BPM with more than 20% Bursts
1.  Chose the menu **Data** and enable **Filter** if its not enabled.
	* On the clomns you gonna see a drop down arrow
1. Click the arrow on **BPM**->**Number filters**->**Between..**
	* On the first enter **210** and on the second **220**
1. Click the arrow on **Stars**->**Number filters**->**Between..**
	* On the first enter **7** and on the second **8**
1. Click the arrow on **Bursts**->**Number filters**->**Greater Than..**
	* On the first enter **20**
1. Click the arrow on **SI**->**Sort Largest to Smallest**
---
#### Example 2: Most Jumpy 6\* DT Beatmaps with no Streams and Deathstreams that are 10.33 AR
1.  Chose the menu **Data** and enable **Filter** if its not enabled.
	* On the clomns you gonna see a drop down arrow
1. Click the arrow on **DT_Stars**->**Number filters**->**Between..**
	* On the first enter **6** and on the second **7**
1. Click the arrow on **Streams**->**Number filters**->**Equals..**
	* On the first enter **0**
1. Click the arrow on **DeathStreams**->**Number filters**->**Equals..**
	* On the first enter **0**
1. Click the arrow on **DT_AR**->**Number filters**->**Equals..**
	* On the first enter **10.33**
1. Click the arrow on **JI**->**Sort Largest to Smallest**

# To Do:
1. [ ] Create a column config file and options file.
1. [ ] Add counter for cut streams
1. [x] Calculate AVG jump Spacing and AVG jump Distance
1. [ ] Option to be able to generate Osu links and Osu Direct links
1. [ ] Expose variables as the stream distance and jump distance for editing in the config
1. [ ] Maybe add optinal support for 1/3 or 1/6th or other uncomon Snap Divisors **(prob wont)**
1. [ ] Figure out a way to have an indicator if the map is ranked/loved/qualified/graveyarded
	* That doesnt require everyone to use thier APIv1 key.
1. [x] Make it so you need to compute once the map and not every time you run the program
1. [x] Include 100+ note stream counter 
1. [x] Make Longest stream counter
## F.A.Q.
1. Why does it run slow ?
	* The program needs to search for all your ".osu" files and then parse all of them to calculate alot of metrics. Most of the time is taken by [ROSU-PP](https://github.com/MaxOhn/rosu-pp) to calculate difficulty and pp for the maps aswel as reading from the disc and parsing. This program scales very well with fast SSD storage and alot of cores *(on my r5 3600x with 860Evo ssd its 2500 maps per second)*
1. When i load the data in Excel the numeric columns like Stars are text ?
	* This can happen for many reasons one being wrong default delimiter in windows. Floating point numbers would expect "," instead of "." . To fix it i know two ways:
		* **Language Settings** -> **Region** -> **Regional format** *(English(Europe) works)*. And then reset Excel.
		* Or in Excel you can mark the column click **Home** around the middle there is a drop down menu usualy writen **General** Switch to: **Number** then go to **Data** and click **Text to Columns** *(should be somewhere on the right)*
## Credits
[ROSU-PP](https://github.com/MaxOhn/rosu-pp) - Beatmap parsing and Difficulty/PP calculations
## Licence
[MIT](https://www.mit.edu/~amini/LICENSE.md)

## Fixed Bugs:
1. **LAST PATTERN OF THE MAP IS NOT BEING CALCULATED => FIXED IN V0.9.1**

[back to the top](#what-is-simple-beatmap-analyzer)