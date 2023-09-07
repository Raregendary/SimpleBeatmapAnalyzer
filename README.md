![Simple_Beatmap_Analyzer(2)](https://github.com/Raregendary/SimpleBeatmapAnalyzer/assets/71941668/8e3e4fe2-021f-4281-ad38-7c25224fcf90)

# What is Simple Beatmap Analyzer ?
###### This is my first rust program
Simple Beatmap Analyzer is a console application that analyzes ".osu" files for Standard gamemode only. After it finishes it outputs the results in a "CSV" file in the same directory as the .exe
## How does it work
It assumes every map is in maped **1/4** and calculates streams/jumps and other patterns aswel as some ratios based on them *(Jumps are asumed every second beat and i will be refering to it as 1/2)*.
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
## How to install
1. Click on releases and download the latest version.
1. Unzip into a folder of your choice.

## How to use it
1. Open the **SimpleBeatmapAnalyzer.exe**
1. Paste the Osu Songs path *(example: D:\osu\Songs)* and click Enter
1. Wait for it to finish and save the data in **results.csv**
1. Open the results.csv in Excel or any other CSV reader/analyzer
1. After you find the map you like simply copy the **MapID** and paste it in the Osu search ingame

## How to load in Excel
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
1. It should auto detect the scope of your table, if not go back to **Method 1**
1. Enable **My table has headers** checkbox and press **Ok**

## How to analyze data in Excel
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
