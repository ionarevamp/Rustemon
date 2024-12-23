#!/bin/bash

cargotarget() {
	#First argument is the desired target/toolchain, the rest are passed directly to cargo
	#(such as '-vvv' or 'build/run')
	local target="$1"
	shift 1
	cargo "+stable-$target" "$@" "--target=$target"
}

HOST_ARCH="${MSYSTEM_CHOST%%-*}"



BUILDRUN="build"

if [ "$(echo $1 | tr '[:upper:]' '[:lower:]')" = "run" ] ; then

	if [ "$HOST_ARCH" = "i686" ] ; then
		BUILDRUN="run"
	else
		echo "Incompatible host architecture. Building only."
	fi
fi

if [ "$(echo $2 | tr '[:upper:]' '[:lower:]')" = "release" ] ; then
	cargotarget "i686-pc-windows-gnu" -vv "$BUILDRUN" "--release"
else
	cargotarget "i686-pc-windows-gnu" -vv "$BUILDRUN"
fi


BUILDRUN="build"

if [ "$(echo $1 | tr '[:upper:]' '[:lower:]')" = "run" ] ; then

	if [ "$HOST_ARCH" = "x86_64" ] ; then
		BUILDRUN="run"
	else
		echo "Incompatible host architecture. Building only."
	fi
fi

if [ "$(echo $2 | tr '[:upper:]' '[:lower:]')" = "release" ] ; then
	cargotarget "x86_64-pc-windows-gnu" -vv "$BUILDRUN" "--release"
else
	cargotarget "x86_64-pc-windows-gnu" -vv "$BUILDRUN"
fi


# TEMPORARY (until audio data is compiled into binary)
for i in target/*/debug target/*/release
do
	cp *.wav "$i"
done
