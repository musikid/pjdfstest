#!/bin/sh
# $FreeBSD: head/tools/regression/pjdfstest/tests/rmdir/14.t 211352 2010-08-15 21:24:17Z pjd $

desc="rmdir returns EROFS if the named file resides on a read-only file system"

dir=`dirname $0`
. ${dir}/../misc.sh

[ "${os}:${fs}" = "FreeBSD:UFS" ] || quick_exit

echo "1..5"

n0=`namegen`
n1=`namegen`

expect 0 mkdir ${n0} 0755
n=`mdconfig -a -n -t malloc -s 1m` || exit
newfs /dev/md${n} >/dev/null || exit
mount /dev/md${n} ${n0}
expect 0 mkdir ${n0}/${n1} 0755
mount -ur /dev/md${n}
expect EROFS rmdir ${n0}/${n1}
mount -uw /dev/md${n}
expect 0 rmdir ${n0}/${n1}
umount /dev/md${n}
mdconfig -d -u ${n} || exit
expect 0 rmdir ${n0}
