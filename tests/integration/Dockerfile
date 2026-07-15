FROM rockylinux:9

# Install required client packages for backup validation, sftp connection, and systemd checks
RUN dnf install -y \
    openssh-clients \
    systemd \
    mariadb \
    postgresql \
    && dnf clean all
