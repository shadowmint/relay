## Notes

### Creating a new test key

    ssh-keygen -t rsa -C "your_email@example.com" -f key
    ssh-keygen -f key.pub -e -m pem > key.pub.pem
    ssh-keygen -f key -y > key.pem