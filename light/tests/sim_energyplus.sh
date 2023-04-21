#!/usr/bin/bash

EPWS=$(ls -d *.epw)
IDFS=$(ls -d *.idf)



for EPW in $EPWS
do 
    for IDF in $IDFS
    do
        dirname=${EPW%.*}_${IDF%.*}
        mkdir $dirname

        energyplus -w $EPW -x -r -d $dirname $IDF 
        # cd $dirname
        # echo 
        # for idf in $(ls | grep .idf)
        # do        
        #     echo Running sim on $dir
        
        # done
        
        # cd ..
    done
done
