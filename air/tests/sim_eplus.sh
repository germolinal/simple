#!/usr/bin/bash

EPW=../wellington.epw



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
