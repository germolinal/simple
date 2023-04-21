#!/usr/bin/bash

EPW=../epw/CHL_Santiago.855740_IWEC.epw



for dir in $(ls -d */)
do 
    cd $dir    
    for idf in $(ls | grep .idf)
    do        
        echo Running sim on $dir
        energyplus -w $EPW -x -r $idf
    done
    
    cd ..
done
