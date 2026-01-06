plugins {
    java
    application
}

group = "com.npcsociety"
version = "1.0.0-SNAPSHOT"

java {
    sourceCompatibility = JavaVersion.VERSION_17
    targetCompatibility = JavaVersion.VERSION_17
}

repositories {
    mavenCentral()
}

val grpcVersion = "1.60.0"
val protobufVersion = "3.25.1"

dependencies {
    // gRPC
    implementation("io.grpc:grpc-netty-shaded:$grpcVersion")
    implementation("io.grpc:grpc-protobuf:$grpcVersion")
    implementation("io.grpc:grpc-stub:$grpcVersion")
    
    // Protobuf
    implementation("com.google.protobuf:protobuf-java:$protobufVersion")
    
    // Generated code (will be available after running buf generate)
    implementation(files("../../gen/java"))
    
    // Annotations for generated code
    compileOnly("org.apache.tomcat:annotations-api:6.0.53")
}

application {
    mainClass.set("com.npcsociety.examples.ExampleClient")
}

